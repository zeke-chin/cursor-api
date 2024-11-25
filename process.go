package main

import (
	"bytes"
	// "encoding/hex"
	// "log"
	"fmt"
)


// // 封装函数，用于将 chunk 转换为 UTF-8 字符串
// function chunkToUtf8String (chunk) {
// 	if (chunk[0] === 0x01 || chunk[0] === 0x02 || (chunk[0] === 0x60 && chunk[1] === 0x0C)) {
// 	  return ''
// 	}
  
// 	console.log('chunk:', Buffer.from(chunk).toString('hex'))
// 	console.log('chunk string:', Buffer.from(chunk).toString('utf-8'))
  
// 	// 去掉 chunk 中 0x0A 以及之前的字符
// 	chunk = chunk.slice(chunk.indexOf(0x0A) + 1)
  
// 	let filteredChunk = []
// 	let i = 0
// 	while (i < chunk.length) {
// 	  // 新的条件过滤：如果遇到连续4个0x00，则移除其之后所有的以 0 开头的字节（0x00 到 0x0F）
// 	  if (chunk.slice(i, i + 4).every(byte => byte === 0x00)) {
// 		i += 4 // 跳过这4个0x00
// 		while (i < chunk.length && chunk[i] >= 0x00 && chunk[i] <= 0x0F) {
// 		  i++ // 跳过所有以 0 开头的字节
// 		}
// 		continue
// 	  }
  
// 	  if (chunk[i] === 0x0C) {
// 		// 遇到 0x0C 时，跳过 0x0C 以及后续的所有连续的 0x0A
// 		i++ // 跳过 0x0C
// 		while (i < chunk.length && chunk[i] === 0x0A) {
// 		  i++ // 跳过所有连续的 0x0A
// 		}
// 	  } else if (
// 		i > 0 &&
// 		chunk[i] === 0x0A &&
// 		chunk[i - 1] >= 0x00 &&
// 		chunk[i - 1] <= 0x09
// 	  ) {
// 		// 如果当前字节是 0x0A，且前一个字节在 0x00 至 0x09 之间，跳过前一个字节和当前字节
// 		filteredChunk.pop() // 移除已添加的前一个字节
// 		i++ // 跳过当前的 0x0A
// 	  } else {
// 		filteredChunk.push(chunk[i])
// 		i++
// 	  }
// 	}
  
// 	// 第二步：去除所有的 0x00 和 0x0C
// 	filteredChunk = filteredChunk.filter((byte) => byte !== 0x00 && byte !== 0x0C)
  
// 	// 去除小于 0x0A 的字节
// 	filteredChunk = filteredChunk.filter((byte) => byte >= 0x0A)
  
// 	const hexString = Buffer.from(filteredChunk).toString('hex')
// 	console.log('hexString:', hexString)
// 	const utf8String = Buffer.from(filteredChunk).toString('utf-8')
// 	console.log('utf8String:', utf8String)
// 	return utf8String
//   }
// func processChunk(chunk []byte) string {
//     // 检查特殊字节开头的情况
//     if len(chunk) > 0 && (chunk[0] == 0x01 || chunk[0] == 0x02 || (len(chunk) > 1 && chunk[0] == 0x60 && chunk[1] == 0x0C)) {
//         return ""
//     }

//     // 打印调试信息
//     fmt.Printf("chunk: %x\n", chunk)
//     fmt.Printf("chunk string: %s\n", string(chunk))

//     // 找到第一个 0x0A 并截取之后的内容
//     index := bytes.IndexByte(chunk, 0x0A)
//     if index != -1 {
//         chunk = chunk[index+1:]
//     }

//     // 创建过滤后的切片
//     filteredChunk := make([]byte, 0, len(chunk))
//     for i := 0; i < len(chunk); {
//         // 检查连续4个0x00的情况
//         if i+4 <= len(chunk) {
//             if chunk[i] == 0x00 && chunk[i+1] == 0x00 && chunk[i+2] == 0x00 && chunk[i+3] == 0x00 {
//                 i += 4
//                 // 跳过所有以0开头的字节
//                 for i < len(chunk) && chunk[i] <= 0x0F {
//                     i++
//                 }
//                 continue
//             }
//         }

//         if chunk[i] == 0x0C {
//             i++
//             // 跳过所有连续的0x0A
//             for i < len(chunk) && chunk[i] == 0x0A {
//                 i++
//             }
//         } else if i > 0 && chunk[i] == 0x0A && chunk[i-1] >= 0x00 && chunk[i-1] <= 0x09 {
//             // 移除前一个字节并跳过当前的0x0A
//             filteredChunk = filteredChunk[:len(filteredChunk)-1]
//             i++
//         } else {
//             filteredChunk = append(filteredChunk, chunk[i])
//             i++
//         }
//     }

//     // 过滤掉0x00和0x0C
//     tempChunk := make([]byte, 0, len(filteredChunk))
//     for _, b := range filteredChunk {
//         if b != 0x00 && b != 0x0C {
//             tempChunk = append(tempChunk, b)
//         }
//     }
//     filteredChunk = tempChunk

//     // 过滤掉小于0x0A的字节
//     tempChunk = make([]byte, 0, len(filteredChunk))
//     for _, b := range filteredChunk {
//         if b >= 0x0A {
//             tempChunk = append(tempChunk, b)
//         }
//     }
//     filteredChunk = tempChunk

//     // 打印调试信息并返回结果
//     fmt.Printf("hexString: %x\n", filteredChunk)
//     result := string(filteredChunk)
//     fmt.Printf("utf8String: %s\n", result)
//     return result
// }

func processChunk(chunk []byte) string {
    // 检查特殊字节开头的情况
    if len(chunk) > 0 && (chunk[0] == 0x01 || chunk[0] == 0x02 || (len(chunk) > 1 && chunk[0] == 0x60 && chunk[1] == 0x0C)) {
        return ""
    }

    // 打印调试信息
    fmt.Printf("chunk: %x\n", chunk)
    fmt.Printf("chunk string: %s\n", string(chunk))

    // 找到第一个 0x0A 并截取之后的内容
    index := bytes.IndexByte(chunk, 0x0A)
    if index != -1 {
        chunk = chunk[index+1:]
    }

    // 创建过滤后的切片
    filteredChunk := make([]byte, 0, len(chunk))
    for i := 0; i < len(chunk); {
        // 检查连续4个0x00的情况
        if i+4 <= len(chunk) {
            allZeros := true
            for j := 0; j < 4; j++ {
                if chunk[i+j] != 0x00 {
                    allZeros = false
                    break
                }
            }
            if allZeros {
                i += 4
                // 跳过所有以0开头的字节
                for i < len(chunk) && chunk[i] <= 0x0F {
                    i++
                }
                continue
            }
        }

        // 保留UTF-8字符
        if chunk[i] >= 0xE0 || (chunk[i] >= 0x20 && chunk[i] <= 0x7F) {
            filteredChunk = append(filteredChunk, chunk[i])
        }
        i++
    }

    // 打印调试信息并返回结果
    fmt.Printf("hexString: %x\n", filteredChunk)
    result := string(filteredChunk)
    fmt.Printf("utf8String: %s\n", result)
    return string(chunk)
}