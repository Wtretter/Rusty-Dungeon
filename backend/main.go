package main

import (
	"encoding/binary"
	"log"
	"net"
)

func handler(connection net.Conn) {
	defer connection.Close()

	message := []byte("Hello, the connection is working")
	bin_len := make([]byte, 2)
	binary.BigEndian.PutUint16(bin_len, uint16(len(message)))
	connection.Write(bin_len)
	connection.Write(message)
}

func main() {
	listener, err := net.Listen("tcp", ":27010")
	if err != nil {
		log.Fatal(err)
	}
	defer listener.Close()

	for {
		// Wait for a connection.
		connection, err := listener.Accept()
		if err != nil {
			log.Fatal(err)
		}

		// Handle connection
		go handler(connection)
	}
}
