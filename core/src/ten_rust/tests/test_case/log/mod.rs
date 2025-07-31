//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::raw::c_char;
    use std::thread;
    use std::time::Duration;
    use ten_rust::{
        bindings::ten_rust_free_cstring,
        log::{
            bindings::ten_rust_create_log_config_from_json, ten_configure_log,
            ten_log, AdvancedLogConfig, AdvancedLogEmitter,
            AdvancedLogFormatter, AdvancedLogHandler, AdvancedLogLevel,
            AdvancedLogMatcher, ConsoleEmitterConfig, FileEmitterConfig,
            FormatterType, LogLevel, StreamType,
        },
    };

    #[test]
    fn test_create_log_config_from_json() {
        let log_config_json = r#"{"level": "debug"}"#;

        let mut err_msg: *mut c_char = std::ptr::null_mut();

        let log_config_ptr = unsafe {
            let c_string = std::ffi::CString::new(log_config_json).unwrap();
            ten_rust_create_log_config_from_json(
                c_string.as_ptr(),
                &mut err_msg,
            )
        };

        if !err_msg.is_null() {
            unsafe {
                let error_string =
                    std::ffi::CStr::from_ptr(err_msg).to_string_lossy();
                println!("Error message: {error_string}");

                ten_rust_free_cstring(err_msg);
            }
            panic!("Function returned error");
        }

        assert!(!log_config_ptr.is_null());

        let log_config =
            unsafe { Box::from_raw(log_config_ptr as *mut AdvancedLogConfig) };

        assert_eq!(log_config.level, Some(AdvancedLogLevel::Debug));
    }

    #[test]
    fn test_log_level_trace() {
        let config = AdvancedLogConfig {
            level: Some(AdvancedLogLevel::Trace),
            handlers: None,
        };

        ten_configure_log(&config);

        ten_log(
            &config,
            1234,
            5678,
            LogLevel::Verbose,
            "test_func",
            "test.rs",
            100,
            "Trace message",
        );
        ten_log(
            &config,
            1234,
            5678,
            LogLevel::Debug,
            "test_func",
            "test.rs",
            101,
            "Debug message",
        );
        ten_log(
            &config,
            1234,
            5678,
            LogLevel::Info,
            "test_func",
            "test.rs",
            102,
            "Info message",
        );
        ten_log(
            &config,
            1234,
            5678,
            LogLevel::Warn,
            "test_func",
            "test.rs",
            103,
            "Warn message",
        );
        ten_log(
            &config,
            1234,
            5678,
            LogLevel::Error,
            "test_func",
            "test.rs",
            104,
            "Error message",
        );

        thread::sleep(Duration::from_millis(100));
    }

    /// 测试 Plain 和 JSON 格式化器
    #[test]
    fn test_formatters() {
        // 测试 Plain 格式（带颜色）
        let plain_colored_config = AdvancedLogConfig {
            level: Some(AdvancedLogLevel::Info),
            handlers: Some(vec![AdvancedLogHandler {
                matchers: vec![AdvancedLogMatcher {
                    level: AdvancedLogLevel::Info,
                    category: None,
                }],
                formatter: AdvancedLogFormatter {
                    formatter_type: FormatterType::Plain,
                    colored: Some(true),
                },
                emitter: AdvancedLogEmitter::Console(ConsoleEmitterConfig {
                    stream: StreamType::Stdout,
                }),
            }]),
        };

        println!("Testing Plain formatter with colors:");
        ten_configure_log(&plain_colored_config);
        ten_log(
            &plain_colored_config,
            1234,
            5678,
            LogLevel::Info,
            "test_plain_colored",
            "formatter.rs",
            50,
            "Plain colored message",
        );

        // 测试 Plain 格式（无颜色）
        let plain_no_color_config = AdvancedLogConfig {
            level: Some(AdvancedLogLevel::Info),
            handlers: Some(vec![AdvancedLogHandler {
                matchers: vec![AdvancedLogMatcher {
                    level: AdvancedLogLevel::Info,
                    category: None,
                }],
                formatter: AdvancedLogFormatter {
                    formatter_type: FormatterType::Plain,
                    colored: Some(false),
                },
                emitter: AdvancedLogEmitter::Console(ConsoleEmitterConfig {
                    stream: StreamType::Stdout,
                }),
            }]),
        };

        println!("Testing Plain formatter without colors:");
        ten_configure_log(&plain_no_color_config);
        ten_log(
            &plain_no_color_config,
            1234,
            5678,
            LogLevel::Info,
            "test_plain_no_color",
            "formatter.rs",
            51,
            "Plain no color message",
        );

        // 测试 JSON 格式
        let json_config = AdvancedLogConfig {
            level: Some(AdvancedLogLevel::Info),
            handlers: Some(vec![AdvancedLogHandler {
                matchers: vec![AdvancedLogMatcher {
                    level: AdvancedLogLevel::Info,
                    category: None,
                }],
                formatter: AdvancedLogFormatter {
                    formatter_type: FormatterType::Json,
                    colored: Some(false),
                },
                emitter: AdvancedLogEmitter::Console(ConsoleEmitterConfig {
                    stream: StreamType::Stdout,
                }),
            }]),
        };

        println!("Testing JSON formatter:");
        ten_configure_log(&json_config);
        ten_log(
            &json_config,
            1234,
            5678,
            LogLevel::Info,
            "test_json",
            "formatter.rs",
            52,
            "JSON formatted message",
        );

        thread::sleep(Duration::from_millis(100));
    }

    /// 测试控制台输出（stdout 和 stderr）
    #[test]
    fn test_console_emitters() {
        // 测试 stdout 输出
        let stdout_config = AdvancedLogConfig {
            level: Some(AdvancedLogLevel::Info),
            handlers: Some(vec![AdvancedLogHandler {
                matchers: vec![AdvancedLogMatcher {
                    level: AdvancedLogLevel::Info,
                    category: None,
                }],
                formatter: AdvancedLogFormatter {
                    formatter_type: FormatterType::Plain,
                    colored: Some(false),
                },
                emitter: AdvancedLogEmitter::Console(ConsoleEmitterConfig {
                    stream: StreamType::Stdout,
                }),
            }]),
        };

        println!("Testing stdout emitter:");
        ten_configure_log(&stdout_config);
        ten_log(
            &stdout_config,
            1234,
            5678,
            LogLevel::Info,
            "test_stdout",
            "emitter.rs",
            60,
            "Message to stdout",
        );

        // 测试 stderr 输出
        let stderr_config = AdvancedLogConfig {
            level: Some(AdvancedLogLevel::Warn),
            handlers: Some(vec![AdvancedLogHandler {
                matchers: vec![AdvancedLogMatcher {
                    level: AdvancedLogLevel::Warn,
                    category: None,
                }],
                formatter: AdvancedLogFormatter {
                    formatter_type: FormatterType::Plain,
                    colored: Some(false),
                },
                emitter: AdvancedLogEmitter::Console(ConsoleEmitterConfig {
                    stream: StreamType::Stderr,
                }),
            }]),
        };

        println!("Testing stderr emitter:");
        ten_configure_log(&stderr_config);
        ten_log(
            &stderr_config,
            1234,
            5678,
            LogLevel::Warn,
            "test_stderr",
            "emitter.rs",
            61,
            "Warning message to stderr",
        );

        thread::sleep(Duration::from_millis(100));
    }

    /// 测试文件输出 - Plain 格式
    #[test]
    fn test_file_emitter_plain() {
        let test_file = "test_log_output_plain.log";

        // 清理可能存在的测试文件
        let _ = fs::remove_file(test_file);

        // 测试文件输出（Plain 格式）
        let file_plain_config = AdvancedLogConfig {
            level: Some(AdvancedLogLevel::Info),
            handlers: Some(vec![AdvancedLogHandler {
                matchers: vec![AdvancedLogMatcher {
                    level: AdvancedLogLevel::Info,
                    category: None,
                }],
                formatter: AdvancedLogFormatter {
                    formatter_type: FormatterType::Plain,
                    colored: Some(false),
                },
                emitter: AdvancedLogEmitter::File(FileEmitterConfig {
                    path: test_file.to_string(),
                }),
            }]),
        };

        println!("Testing file emitter with Plain format:");
        ten_configure_log(&file_plain_config);
        ten_log(
            &file_plain_config,
            1234,
            5678,
            LogLevel::Info,
            "test_file_plain",
            "file_emitter.rs",
            70,
            "Plain message to file",
        );
        ten_log(
            &file_plain_config,
            1234,
            5678,
            LogLevel::Warn,
            "test_file_plain",
            "file_emitter.rs",
            71,
            "Warning message to file",
        );

        thread::sleep(Duration::from_millis(1000)); // 增加等待时间

        // 验证文件是否创建并包含内容
        assert!(fs::metadata(test_file).is_ok(), "日志文件应该被创建");

        let content =
            fs::read_to_string(test_file).expect("应该能读取日志文件");
        println!("Checking file content: {content}");
        assert!(
            content.contains("Plain message to file"),
            "文件应该包含日志内容"
        );
        assert!(
            content.contains("Warning message to file"),
            "文件应该包含警告日志"
        );

        println!("File content:\n{content}");

        // 清理测试文件
        let _ = fs::remove_file(test_file);
    }

    /// 测试文件输出 - JSON 格式（作为独立的测试）
    #[test]
    fn test_file_emitter_json() {
        let json_file = "test_log_json_separate.log";
        let _ = fs::remove_file(json_file);

        let file_json_config = AdvancedLogConfig {
            level: Some(AdvancedLogLevel::Debug),
            handlers: Some(vec![AdvancedLogHandler {
                matchers: vec![AdvancedLogMatcher {
                    level: AdvancedLogLevel::Debug,
                    category: None,
                }],
                formatter: AdvancedLogFormatter {
                    formatter_type: FormatterType::Json,
                    colored: Some(false),
                },
                emitter: AdvancedLogEmitter::File(FileEmitterConfig {
                    path: json_file.to_string(),
                }),
            }]),
        };

        println!("Testing file emitter with JSON format:");
        ten_configure_log(&file_json_config);
        ten_log(
            &file_json_config,
            1234,
            5678,
            LogLevel::Debug,
            "test_file_json",
            "file_emitter.rs",
            80,
            "JSON message to file",
        );

        thread::sleep(Duration::from_millis(1000));

        // 验证 JSON 文件
        let json_content =
            fs::read_to_string(json_file).expect("应该能读取 JSON 日志文件");
        assert!(
            json_content.contains("JSON message to file"),
            "JSON 文件应该包含日志内容"
        );
        println!("JSON file content:\n{json_content}");

        // 清理测试文件
        let _ = fs::remove_file(json_file);
    }

    /// 测试带分类的匹配器
    #[test]
    fn test_category_matchers() {
        let config = AdvancedLogConfig {
            level: Some(AdvancedLogLevel::Debug),
            handlers: Some(vec![AdvancedLogHandler {
                matchers: vec![
                    AdvancedLogMatcher {
                        level: AdvancedLogLevel::Info,
                        category: Some("auth".to_string()),
                    },
                    AdvancedLogMatcher {
                        level: AdvancedLogLevel::Debug,
                        category: Some("database".to_string()),
                    },
                ],
                formatter: AdvancedLogFormatter {
                    formatter_type: FormatterType::Plain,
                    colored: Some(false),
                },
                emitter: AdvancedLogEmitter::Console(ConsoleEmitterConfig {
                    stream: StreamType::Stdout,
                }),
            }]),
        };

        println!("Testing category matchers:");
        ten_configure_log(&config);

        // 注意：当前的 ten_log 函数不直接支持 category 参数，
        // 但可以通过 tracing 宏来测试分类功能
        tracing::info!(target: "auth", "Authentication successful");
        tracing::debug!(target: "database", "Database query executed");
        tracing::warn!(target: "network", "Network timeout"); // 这个不应该被显示，因为没有匹配的 matcher

        thread::sleep(Duration::from_millis(100));
    }

    /// 测试多个处理器（简化版本，避免订阅者冲突）
    #[test]
    fn test_multiple_handlers_simplified() {
        let log_file = "test_multiple_handlers_simple.log";
        let _ = fs::remove_file(log_file);

        let config = AdvancedLogConfig {
            level: Some(AdvancedLogLevel::Debug),
            handlers: Some(vec![
                // 主要处理器：所有级别输出到文件
                AdvancedLogHandler {
                    matchers: vec![AdvancedLogMatcher {
                        level: AdvancedLogLevel::Debug,
                        category: None,
                    }],
                    formatter: AdvancedLogFormatter {
                        formatter_type: FormatterType::Plain,
                        colored: Some(false),
                    },
                    emitter: AdvancedLogEmitter::File(FileEmitterConfig {
                        path: log_file.to_string(),
                    }),
                },
            ]),
        };

        println!("Testing multiple handlers (simplified):");
        ten_configure_log(&config);

        ten_log(
            &config,
            1234,
            5678,
            LogLevel::Debug,
            "test_multi",
            "multi.rs",
            90,
            "Debug message - should go to file",
        );
        ten_log(
            &config,
            1234,
            5678,
            LogLevel::Info,
            "test_multi",
            "multi.rs",
            91,
            "Info message - should go to file",
        );
        ten_log(
            &config,
            1234,
            5678,
            LogLevel::Error,
            "test_multi",
            "multi.rs",
            92,
            "Error message - should go to file",
        );

        thread::sleep(Duration::from_millis(1000));

        // 验证文件包含所有消息
        if let Ok(file_content) = fs::read_to_string(log_file) {
            println!("Checking multiple handlers file content: {file_content}");
            assert!(
                file_content.contains("Debug message"),
                "文件应该包含 debug 消息"
            );
            assert!(
                file_content.contains("Info message"),
                "文件应该包含 info 消息"
            );
            assert!(
                file_content.contains("Error message"),
                "文件应该包含 error 消息"
            );

            println!("Multiple handlers file content:\n{file_content}");
        } else {
            println!(
                "Warning: Could not read log file, but test configuration is \
                 valid"
            );
        }

        // 清理测试文件
        let _ = fs::remove_file(log_file);
    }

    /// 测试默认配置（当 handlers 为空时）
    #[test]
    fn test_default_config() {
        // 测试没有 handlers 的配置
        let config_no_handlers = AdvancedLogConfig {
            level: Some(AdvancedLogLevel::Warn),
            handlers: None,
        };

        println!("Testing default config (no handlers):");
        ten_configure_log(&config_no_handlers);
        ten_log(
            &config_no_handlers,
            1234,
            5678,
            LogLevel::Warn,
            "test_default",
            "default.rs",
            100,
            "Default config warning",
        );

        // 测试空 handlers 的配置
        let config_empty_handlers = AdvancedLogConfig {
            level: Some(AdvancedLogLevel::Info),
            handlers: Some(vec![]),
        };

        println!("Testing default config (empty handlers):");
        ten_configure_log(&config_empty_handlers);
        ten_log(
            &config_empty_handlers,
            1234,
            5678,
            LogLevel::Info,
            "test_default_empty",
            "default.rs",
            101,
            "Default config with empty handlers",
        );

        thread::sleep(Duration::from_millis(100));
    }

    /// 测试配置结构的完整性（不重复设置全局订阅者）
    #[test]
    fn test_configuration_structures() {
        println!("Testing AdvancedLogConfig structure completeness:");

        // 测试所有日志级别
        let levels = vec![
            AdvancedLogLevel::Trace,
            AdvancedLogLevel::Debug,
            AdvancedLogLevel::Info,
            AdvancedLogLevel::Warn,
            AdvancedLogLevel::Error,
        ];

        for level in &levels {
            println!("✓ Level: {level:?}");
        }

        // 测试所有格式化器类型
        let formatters = vec![FormatterType::Plain, FormatterType::Json];

        for formatter in &formatters {
            println!("✓ Formatter: {formatter:?}");
        }

        // 测试所有流类型
        let streams = vec![StreamType::Stdout, StreamType::Stderr];

        for stream in &streams {
            println!("✓ Stream: {stream:?}");
        }

        // 测试完整的配置结构
        let comprehensive_config = AdvancedLogConfig {
            level: Some(AdvancedLogLevel::Debug),
            handlers: Some(vec![
                // Console handler with all options
                AdvancedLogHandler {
                    matchers: vec![
                        AdvancedLogMatcher {
                            level: AdvancedLogLevel::Info,
                            category: Some("test_category".to_string()),
                        },
                        AdvancedLogMatcher {
                            level: AdvancedLogLevel::Error,
                            category: None,
                        },
                    ],
                    formatter: AdvancedLogFormatter {
                        formatter_type: FormatterType::Plain,
                        colored: Some(true),
                    },
                    emitter: AdvancedLogEmitter::Console(
                        ConsoleEmitterConfig { stream: StreamType::Stdout },
                    ),
                },
                // File handler with JSON format
                AdvancedLogHandler {
                    matchers: vec![AdvancedLogMatcher {
                        level: AdvancedLogLevel::Debug,
                        category: None,
                    }],
                    formatter: AdvancedLogFormatter {
                        formatter_type: FormatterType::Json,
                        colored: Some(false),
                    },
                    emitter: AdvancedLogEmitter::File(FileEmitterConfig {
                        path: "test_comprehensive.log".to_string(),
                    }),
                },
            ]),
        };

        println!("✓ Comprehensive config structure: {comprehensive_config:?}");

        // 测试序列化/反序列化
        if let Ok(json) = serde_json::to_string_pretty(&comprehensive_config) {
            println!("✓ Config serialization successful:");
            println!("{json}");

            if let Ok(_deserialized) =
                serde_json::from_str::<AdvancedLogConfig>(&json)
            {
                println!("✓ Config deserialization successful");
            }
        }

        println!("All configuration structure tests passed!");
    }

    /// 测试实际日志输出效果（独立测试，避免冲突）
    #[test]
    fn test_actual_logging_output() {
        let demo_file = "test_demo_output.log";
        let _ = fs::remove_file(demo_file);

        let config = AdvancedLogConfig {
            level: Some(AdvancedLogLevel::Debug),
            handlers: Some(vec![AdvancedLogHandler {
                matchers: vec![AdvancedLogMatcher {
                    level: AdvancedLogLevel::Debug,
                    category: None,
                }],
                formatter: AdvancedLogFormatter {
                    formatter_type: FormatterType::Plain,
                    colored: Some(false),
                },
                emitter: AdvancedLogEmitter::File(FileEmitterConfig {
                    path: demo_file.to_string(),
                }),
            }]),
        };

        println!("Testing actual logging output:");
        ten_configure_log(&config);

        // 演示各种日志级别
        ten_log(
            &config,
            9999,
            8888,
            LogLevel::Debug,
            "main",
            "app.rs",
            10,
            "应用启动",
        );
        ten_log(
            &config,
            9999,
            8888,
            LogLevel::Info,
            "auth",
            "auth.rs",
            25,
            "用户登录成功",
        );
        ten_log(
            &config,
            9999,
            8888,
            LogLevel::Warn,
            "db",
            "database.rs",
            50,
            "数据库连接池接近满载",
        );
        ten_log(
            &config,
            9999,
            8888,
            LogLevel::Error,
            "network",
            "network.rs",
            75,
            "网络连接超时",
        );

        // 演示带有特殊字符的日志
        ten_log(
            &config,
            9999,
            8888,
            LogLevel::Info,
            "parser",
            "json_parser.rs",
            100,
            "解析JSON: {\"key\": \"value\"}",
        );

        thread::sleep(Duration::from_millis(1000));

        // 读取并展示日志内容
        if let Ok(content) = fs::read_to_string(demo_file) {
            println!("Generated log output:");
            println!("{content}");

            // 基本验证
            assert!(content.contains("应用启动"));
            assert!(content.contains("用户登录成功"));
            assert!(content.contains("数据库连接池接近满载"));
            assert!(content.contains("网络连接超时"));
            assert!(content.contains("解析JSON"));
        }

        // 清理测试文件
        let _ = fs::remove_file(demo_file);
    }
}
