package main

import (

	"go-capi/handlers"

	"github.com/gin-contrib/cors"
	"github.com/gin-gonic/gin"
	"github.com/joho/godotenv"
)

func main() {
	// 加载环境变量
	godotenv.Load()

	r := gin.Default()

	// 配置CORS
	r.Use(cors.New(cors.Config{
		AllowOrigins:     []string{"*"},
		AllowMethods:     []string{"GET", "POST",},
		AllowHeaders:     []string{"*"},
		AllowCredentials: true,
	}))

	// 注册路由
	r.POST("/v1/chat/completions", handlers.ChatCompletions)
	r.GET("/models", handlers.GetModels)

	// 获取端口号
	// port := os.Getenv("PORT")
	port := "3001"

	r.Run(":" + port)
}