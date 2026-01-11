package main

import (
	"encoding/binary"
	"log"
	"net"
)

func handler(connection net.Conn) {
	defer connection.Close()
	for {
		message, err := recv_string(connection)
		if err != nil {
			println(err)
			return
		}

		println("Message Recv: ", message)

		err = send_string(connection, message)
		if err != nil {
			println(err)
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
