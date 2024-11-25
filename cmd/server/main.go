package main

import (
	"log"
	"os"

	"github.com/joho/godotenv"
	"cursor-api-proxy/internal/api"
)

func main() {
	if err := godotenv.Load(); err != nil {
		log.Println("Warning: Error loading .env file")
	}

	server := api.NewServer()
	
	port := os.Getenv("PORT")
	if port == "" {
		port = "3000"
	}
	
	log.Printf("服务器运行在端口 %s\n", port)
	server.Run(":" + port)
} 