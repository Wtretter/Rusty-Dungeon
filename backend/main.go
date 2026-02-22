package main

import (
	"database/sql"
	"encoding/binary"
	"encoding/json"
	"fmt"
	"log"
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

func handler(connection net.Conn) {
	defer connection.Close()

	var username string
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
			username = data["username"].(string)
			if username == "" {
				return
			}

			DatabaseConnector()

			fmt.Println("User [", username, "] has logged in")

			state = StateShop
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

func DatabaseConnector() {

	// why does this return nil?
	pw, found := os.LookupEnv("POSTGRES_PASSWORD")
	fmt.Println("pw:", pw)
	fmt.Println("ENV found?", found)

	connector, err := pq.NewConnector("host=backend-db-1 user=root password=TempPasswordReplaceLater sslmode=disable dbname=db-1")
	if err != nil {
		log.Fatalf("could not create connector: %v", err)
	}

	db := sql.OpenDB(connector)
	fmt.Println("ping:", db.Ping())
	defer db.Close()

	q, err := db.Query("SELECT name FROM enemies")

	fmt.Println("A:", q)

	if err != nil {
		fmt.Println("err:", err)
	}
}

func main() {

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
		go handler(connection)
	}
}
