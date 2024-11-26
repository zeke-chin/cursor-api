package utils

import (
	"bytes"
	"encoding/hex"
	"fmt"
	"strings"
	"unicode/utf8"
)

func StringToHex(text string, modelName string) ([]byte, error) {
	textBytes := []byte(text)
	textLength := len(textBytes)

	const (
		FIXED_HEADER = 2
		SEPARATOR    = 1
	)

	modelNameBytes := []byte(modelName)
	FIXED_SUFFIX_LENGTH := 0xA3 + len(modelNameBytes)

	// 计算第一个长度字段
	var textLengthField1 string
	var textLengthFieldSize1 int
	if textLength < 128 {
		textLengthField1 = fmt.Sprintf("%02x", textLength)
		textLengthFieldSize1 = 1
	} else {
		lowByte1 := (textLength & 0x7F) | 0x80
		highByte1 := (textLength >> 7) & 0xFF
		textLengthField1 = fmt.Sprintf("%02x%02x", lowByte1, highByte1)
		textLengthFieldSize1 = 2
	}

	// 计算基础长度字段
	baseLength := textLength + 0x2A
	var textLengthField string
	var textLengthFieldSize int
	if baseLength < 128 {
		textLengthField = fmt.Sprintf("%02x", baseLength)
		textLengthFieldSize = 1
	} else {
		lowByte := (baseLength & 0x7F) | 0x80
		highByte := (baseLength >> 7) & 0xFF
		textLengthField = fmt.Sprintf("%02x%02x", lowByte, highByte)
		textLengthFieldSize = 2
	}

	// 计算总消息长度
	messageTotalLength := FIXED_HEADER + textLengthFieldSize + SEPARATOR + textLengthFieldSize1 + textLength + FIXED_SUFFIX_LENGTH

	modelNameHex := strings.ToUpper(hex.EncodeToString(modelNameBytes))
	modelNameLengthHex := fmt.Sprintf("%02X", len(modelNameBytes))

	hexString := fmt.Sprintf(
		"%010x"+
			"12"+
			"%s"+
			"0a"+
			"%s"+
			"%x"+
			"10016a2432343163636435662d393162612d343131382d393239612d3936626330313631626432612"+
			"2002a132f643a2f6964656150726f2f656475626f73733a1e0a"+
			"%s"+
			"%s"+
			"22004a"+
			"2461383761396133342d323164642d343863372d623434662d616636633365636536663765"+
			"680070007a2436393737376535612d386332642d343835342d623564392d653062623232336163303061"+
			"800101b00100c00100e00100e80100",
		messageTotalLength,
		textLengthField,
		textLengthField1,
		textBytes,
		modelNameLengthHex,
		modelNameHex,
	)

	hexString = strings.ToLower(hexString)

	return hex.DecodeString(hexString)
}

func ChunkToUTF8String(chunk []byte) string {
	// 基础检查
	if len(chunk) < 2 {
		return ""
	}

	if chunk[0] == 0x01 || chunk[0] == 0x02 || (chunk[0] == 0x60 && chunk[1] == 0x0C) {
		return ""
	}

	// 修改调试输出格式
	// fmt.Printf("chunk length: %d hex: %x\n", len(chunk), chunk)
	fmt.Printf("chunk length: %d\n", len(chunk), )

	// 去掉0x0A之前的所有字节
	if idx := bytes.IndexByte(chunk, 0x0A); idx != -1 {
		chunk = chunk[idx+1:]
	}

	// 修改过滤逻辑，将过滤步骤分开
	filteredChunk := make([]byte, 0, len(chunk))
	i := 0
	for i < len(chunk) {
		// 检查连续的0x00
		if i+4 <= len(chunk) && allZeros(chunk[i:i+4]) {
			i += 4
			for i < len(chunk) && chunk[i] <= 0x0F {
				i++
			}
			continue
		}

		if chunk[i] == 0x0C {
			i++
			for i < len(chunk) && chunk[i] == 0x0A {
				i++
			}
		} else {
			filteredChunk = append(filteredChunk, chunk[i])
			i++
		}
	}

	// 最后统一过滤特定字节
	finalFiltered := make([]byte, 0, len(filteredChunk))
	for _, b := range filteredChunk {
		if b != 0x00 && b != 0x0C {
			finalFiltered = append(finalFiltered, b)
		}
	}

	if len(finalFiltered) == 0 {
		return ""
	}

	// 添加错误处理
	result := strings.TrimSpace(string(finalFiltered))
	if !utf8.Valid(finalFiltered) {
		fmt.Printf("Error: Invalid UTF-8 sequence\n")
		return ""
	}

	fmt.Printf("decoded result: %s\n", result)
	return result
}

// 辅助函数检查连续的零字节
func allZeros(data []byte) bool {
	for _, b := range data {
		if b != 0x00 {
			return false
		}
	}
	return true
} 