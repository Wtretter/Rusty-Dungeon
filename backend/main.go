package main

import (
	"database/sql"
	"encoding/binary"
	"encoding/json"
	"fmt"
	"log"
	"math/rand"
	"net"
	"os"

	"github.com/lib/pq"
)

const (
	StateStartup = 0
	StateLogin   = 1
	StateShop    = 10
	StateRun     = 11
)

type Enemy struct {
	Name       string
	Hitpoints  int
	Damage     int
	Resistance int
	Crit       int
	Value      int
}

type Player struct {
	Id         int
	Username   string
	Money      int
	Hitpoints  int
	Damage     int
	Luck       int
	Resistance int
	Crit       int
}

type DamageEvent struct {
	Amount   int
	Crit     bool
	Resisted bool
	Incoming bool
}

type FightRecord struct {
	Enemy        Enemy
	DamageEvents []DamageEvent
	CoinEarned   int
}

type RunRecord struct {
	Fights []FightRecord
}

func (player *Player) PrintStats() {
	fmt.Println(
		"Stats:",
		"\nID:", player.Id,
		"\nUsername:", player.Username,
		"\nMoney", player.Money,
		"\nHitpoints", player.Hitpoints,
		"\nDamage", player.Damage,
		"\nLuck", player.Luck,
		"\nResistance", player.Resistance,
		"\nCrit", player.Crit)
}

func (player *Player) Save(db *sql.DB) error {
	_, err := db.Exec(
		"UPDATE users SET username = $1, money = $2, hitpoints = $3, damage = $4, luck = $5, resistance = $6, crit = $7 WHERE id = $8",
		player.Username, player.Money, player.Hitpoints, player.Damage, player.Luck, player.Resistance, player.Crit, player.Id,
	)
	return err
}

func (player *Player) Fight(enemy Enemy) FightRecord {
	var fight_record FightRecord
	fight_record.Enemy = enemy

	for player.Hitpoints > 0 {
		var outgoing_damage_event DamageEvent
		outgoing_damage_event.Incoming = false
		outgoing_damage_event.Amount = player.Damage
		if player.Crit >= rand.Intn(100) {
			outgoing_damage_event.Amount *= 5
			outgoing_damage_event.Crit = true
		}
		if enemy.Resistance >= rand.Intn(100) {
			outgoing_damage_event.Amount /= 2
			outgoing_damage_event.Resisted = true
		}
		enemy.Hitpoints -= outgoing_damage_event.Amount
		fight_record.DamageEvents = append(fight_record.DamageEvents, outgoing_damage_event)

		if enemy.Hitpoints <= 0 {
			coins := enemy.Value
			if player.Luck >= rand.Intn(100) {
				coins *= 5
			}
			fight_record.CoinEarned = coins
			player.Money += coins
			break
		}
		var incoming_damage_event DamageEvent
		incoming_damage_event.Incoming = true
		incoming_damage_event.Amount = enemy.Damage

		if enemy.Crit >= rand.Intn(100) {
			incoming_damage_event.Amount *= 5
			incoming_damage_event.Crit = true
		}
		if player.Resistance >= rand.Intn(100) {
			incoming_damage_event.Amount /= 2
			incoming_damage_event.Resisted = true
		}
		player.Hitpoints -= incoming_damage_event.Amount
		fight_record.DamageEvents = append(fight_record.DamageEvents, incoming_damage_event)
	}
	return fight_record
}

func get_enemies(db *sql.DB) ([]Enemy, error) {
	rows, err := db.Query("SELECT name, hitpoints, damage, resistance, crit, Value FROM enemies")
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var enemies []Enemy

	for rows.Next() {
		var enemy Enemy
		if err := rows.Scan(&enemy.Name, &enemy.Hitpoints, &enemy.Damage,
			&enemy.Resistance, &enemy.Crit, &enemy.Value); err != nil {
			return enemies, err
		}
		enemies = append(enemies, enemy)
	}
	if err = rows.Err(); err != nil {
		return enemies, err
	}
	return enemies, nil
}

func handler(connection net.Conn, db *sql.DB) {
	defer connection.Close()

	var player Player

	state := StateStartup

	for {
		message, err := recv_string(connection)
		if err != nil {
			fmt.Println("Connection to client dropped, err:", err)
			return
		}

		// fmt.Println("Message Recv: ", message)

		var data map[string]any

		err = json.Unmarshal([]byte(message), &data)
		if err != nil {
			fmt.Println("Failed to parse message as JSON. err:", err)
			return
		}
		fmt.Println(data)

		message_type := data["message_type"]

		// fmt.Println("message type:", message_type)

		if message_type == "login" {
			player.Username = data["username"].(string)
			if player.Username == "" {
				println("Username may not be empty")
				return
			}

			row := db.QueryRow("SELECT * FROM users WHERE username=$1", player.Username)

			err := row.Scan(&player.Id, &player.Username, &player.Money, &player.Hitpoints, &player.Damage, &player.Luck, &player.Resistance, &player.Crit)

			if err != nil {
				fmt.Println("err:", err)
				return
				// TODO add registration message here
			}

			fmt.Println("User [", player.Username, "-", player.Id, "] has logged in")

			player.PrintStats()

			state = StateShop
		}

		if message_type == "register" {
			player.Username = data["username"].(string)
			if player.Username == "" {
				println("Username may not be empty")
				return
			}
			_, err := db.Exec("INSERT INTO users (username, money, hitpoints, damage, luck, resistance, crit) ValueS ($1, 0, 10, 1, 1, 0, 1)", player.Username)

			if err != nil {
				fmt.Println("registration error:", err)
				return
			}
			state = StateShop
		}

		if message_type == "run" {
			var run_loop int
			var run_record RunRecord
			initial_health := player.Hitpoints

			// enemies_killed := 0

			enemies, err := get_enemies(db)

			if err != nil {
				fmt.Println("get_enemies returned error:", err)
				return
			}

			for player.Hitpoints > 0 && run_loop < 10000 {
				fmt.Println("loop:", run_loop)
				rand := rand.Intn(3)
				enemy := enemies[rand]
				fmt.Println("selected enemy:", enemy)

				fight_record := player.Fight(enemy)
				run_record.Fights = append(run_record.Fights, fight_record)
				if player.Hitpoints <= 0 {
					player.Hitpoints = initial_health
					player.Save(db)
					break
				}

				run_loop++
			}

			json_run_record, err := json.Marshal(run_record)
			if err != nil {
				fmt.Println("couldnt JSON", err)
				break
			}
			string_run_record := string(json_run_record)
			send_string(connection, string_run_record)
			continue
		}

		err = send_uint16(connection, uint16(state))
		if err != nil {
			fmt.Println("Send failed, err:", err)
			return
		}
	}
}

func recv_string(connection net.Conn) (string, error) {
	buffer := make([]byte, 2)
	var err error

	_, err = connection.Read(buffer)
	if err != nil {
		return "", err
	}

	recv_len := binary.BigEndian.Uint16(buffer)
	buffer = make([]byte, recv_len)

	_, err = connection.Read(buffer)
	if err != nil {
		return "", err
	}

	return string(buffer), nil

}

func send_string(connection net.Conn, message string) error {
	var err error
	bin_message := []byte(message)
	bin_len := make([]byte, 2)

	binary.BigEndian.PutUint16(bin_len, uint16(len(bin_message)))
	_, err = connection.Write(bin_len)
	if err != nil {
		return err
	}
	_, err = connection.Write(bin_message)
	if err != nil {
		return err
	}
	return nil
}

func send_uint16(connection net.Conn, number uint16) error {
	var err error
	bin_len := make([]byte, 2)

	binary.BigEndian.PutUint16(bin_len, number)
	_, err = connection.Write(bin_len)
	if err != nil {
		return err
	}
	return nil
}

func DatabaseConnector() *sql.DB {

	pw, found := os.LookupEnv("POSTGRES_PASSWORD")

	if !found {
		fmt.Println("Could not pull DB password from ENV")
		return nil
	}

	connector, err := pq.NewConnector(fmt.Sprintf("host=db user=root password=%s sslmode=disable dbname=db-1", pw))
	if err != nil {
		log.Fatalf("could not create connector: %v", err)
		return nil
	}

	db := sql.OpenDB(connector)

	err = db.Ping()

	if err != nil {
		fmt.Println("err:", err)
		return nil
	}

	return db

}

func main() {

	db := DatabaseConnector()

	if db == nil {
		println("could not connect to Database")
		return
	}

	listener, err := net.Listen("tcp", ":27010")
	if err != nil {
		log.Fatal(err)
	}
	defer listener.Close()
	println("Server Up")

	for {
		// Wait for a connection.
		connection, err := listener.Accept()
		if err != nil {
			log.Fatal(err)
		}
		println("Connection established")
		// Handle connection
		go handler(connection, db)
	}
}
