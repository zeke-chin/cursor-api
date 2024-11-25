package main

import (
	"bytes"
	"encoding/hex"
	"fmt"
	"strings"
)

func stringToHex(str, modelName string) []byte {
	inputBytes := []byte(str)
	byteLength := len(inputBytes)

	const (
		FIXED_HEADER = 2
		SEPARATOR    = 1
	)
	
	FIXED_SUFFIX_LENGTH := 0xA3 + len(modelName)

	// 计算文本长度字段
	var textLengthField1, textLengthFieldSize1 int
	if byteLength < 128 {
		textLengthField1 = byteLength
		textLengthFieldSize1 = 1
	} else {
		lowByte1 := (byteLength & 0x7F) | 0x80
		highByte1 := (byteLength >> 7) & 0xFF
		textLengthField1 = (highByte1 << 8) | lowByte1
		textLengthFieldSize1 = 2
	}

	// 计算基础长度
	baseLength := byteLength + 0x2A
	var textLengthField, textLengthFieldSize int
	if baseLength < 128 {
		textLengthField = baseLength
		textLengthFieldSize = 1
	} else {
		lowByte := (baseLength & 0x7F) | 0x80
		highByte := (baseLength >> 7) & 0xFF
		textLengthField = (highByte << 8) | lowByte
		textLengthFieldSize = 2
	}

	// 计算总消息长度
	messageTotalLength := FIXED_HEADER + textLengthFieldSize + SEPARATOR +
		textLengthFieldSize1 + byteLength + FIXED_SUFFIX_LENGTH

	var buf bytes.Buffer
	
	// 写入消息长度
	fmt.Fprintf(&buf, "%010x", messageTotalLength)
	
	// 写入固定头部
	buf.WriteString("12")
	
	// 写入长度字段
	fmt.Fprintf(&buf, "%02x", textLengthField)
	
	buf.WriteString("0A")
	fmt.Fprintf(&buf, "%02x", textLengthField1)
	
	// 写入消息内容
	buf.WriteString(hex.EncodeToString(inputBytes))
	
	// 写入固定后缀
	buf.WriteString("10016A2432343163636435662D393162612D343131382D393239612D3936626330313631626432612")
	buf.WriteString("2002A132F643A2F6964656150726F2F656475626F73733A1E0A")
	
	// 写入模型名称长度和内容
	fmt.Fprintf(&buf, "%02X", len(modelName))
	buf.WriteString(strings.ToUpper(hex.EncodeToString([]byte(modelName))))
	
	// 写入剩余固定内容
	buf.WriteString("22004A")
	buf.WriteString("2461383761396133342D323164642D343863372D623434662D616636633365636536663765")
	buf.WriteString("680070007A2436393337376535612D386332642D343835342D623564392D653062623232336163303061")
	buf.WriteString("800101B00100C00100E00100E80100")

	hexBytes, _ := hex.DecodeString(strings.ToUpper(buf.String()))
	return hexBytes
} 