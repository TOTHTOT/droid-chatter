# droid-chatter

一个 Rust 库，用于生成星球大战机器人（BD-1、D-O、Astromech 等）的音效。

> 本项目是对 [node-ttastromech](https://github.com/wilburforce83/node-ttastromech) 的 Rust 实现，使用 rodio/cpal 替代 Node.js 的 speaker 库实现音频播放。

## 支持的机器人

| 机器人 | 类型 | 说明 |
|--------|------|------|
| **BD-1** | 字母拼接 | 来自《星球大战绝地：陨落的武士团》，支持情绪（happy/sad/angry） |
| **D-O** | 预录音 | 完整的短语音效 |
| **Astromech** | 字母拼接 | 经典 R2-D2 风格（R2、C-3PO 等） |
| **BB-8** | 预录音 | BB-8 专属音效 |
| **Mouse Droid** | 预录音 | MSE-6 鼠标机器人 |
| **Chopper** | 预录音 | 协议机器人 Chopper |
| **Probe Droid** | 预录音 | 帝国探测机器人 |
| **R2** | 预录音 | R2 系列机器人 |

## 安装

```toml
[dependencies]
droid-chatter = "0.1.0"
```

## 自动下载音效

首次使用时自动从 npm 下载音效文件（约 300MB），下载后缓存到本地：

```rust
use droid_chatter::{setup_sounds, DroidChatter, Mood};

fn main() {
    // 自动下载音效（如果不存在）
    setup_sounds("./sounds").unwrap();

    let chatter = DroidChatter::new("./sounds").unwrap();
    chatter.bd1("hello", Mood::Happy).unwrap();
}
```

## 快速开始

```rust
use droid_chatter::{DroidChatter, Mood};

fn main() {
    let chatter = DroidChatter::new("sounds").unwrap();

    // BD-1 说话
    chatter.bd1("hello", Mood::Happy).unwrap();

    // D-O 说话
    chatter.do_("hello1").unwrap();

    // 生成 WAV 文件
    chatter.bd1_to_file("hello", Mood::Happy, "output.wav").unwrap();
}
```

## API 详解

### 初始化

```rust
use droid_chatter::setup_sounds;

// 首次运行自动下载音效到指定目录
setup_sounds("./sounds").unwrap();

let chatter = DroidChatter::new("./sounds").unwrap();
```

### 播放音效

#### BD-1（字母拼接 + 情绪）

```rust
// 指定情绪说话
chatter.bd1("hello", Mood::Happy).unwrap();    // 开心
chatter.bd1("hello", Mood::Sad).unwrap();       // 悲伤
chatter.bd1("hello", Mood::Angry).unwrap();     // 愤怒

// 随机音效
chatter.bd1_random(30, Mood::Happy).unwrap();   // 生成 30 字符随机音效
```

#### D-O（预录音短语）

```rust
// 播放指定短语
chatter.do_("hello1").unwrap();
chatter.do_("batterycharged").unwrap();
chatter.do_("iamdo").unwrap();

// 播放随机短语
chatter.do_("random").unwrap();
```

可用的 D-O 短语：
- `hello1`, `hello2`, `hello3`
- `iamdo`
- `batterycharged`
- `friendsahead`
- `imissher`
- `nothanks1`, `nothanks2`
- `sad`
- `sosorryshesgone`
- `squeekgone`, `squeeky`
- `whatisthat`

#### Astromech（R2-D2 风格）

```rust
// 播放
chatter.astro("r2d2").unwrap();
chatter.astro("beep").unwrap();

// 随机音效
chatter.astro_random(30).unwrap();
```

#### 预录音机器人

```rust
chatter.bb8("BB-8 excited 01.wav").unwrap();
chatter.chopper("001_startup.wav").unwrap();
chatter.mouse("SW01_Vehicles_MSE6_MouseDroid_Alert_VAR_01 0 0 0.wav").unwrap();
chatter.probe("complete.wav").unwrap();
chatter.r2("1.wav").unwrap();
```

### 生成 WAV 文件

```rust
// BD-1 生成到文件
chatter.bd1_to_file("hello", Mood::Happy, "output/bd1_hello.wav").unwrap();

// Astromech 生成到文件
chatter.astro_to_file("r2d2", "output/astro_r2d2.wav").unwrap();
```

### 获取原始音频数据（用于 cpal 播放）

```rust
use droid_chatter::{DroidChatter, DroidType, Mood};

let chatter = DroidChatter::new("sounds").unwrap();

// 获取音频数据
let audio = chatter.get_audio_data("hello", DroidType::Bd1, Some(Mood::Happy)).unwrap();

// AudioData 结构:
println!("samples: {} samples", audio.samples.len());
println!("sample_rate: {} Hz", audio.sample_rate);
println!("channels: {}", audio.channels);
println!("frames: {}", audio.frames());

// 便捷方法
let bd1_audio = chatter.bd1_audio("hello", Mood::Happy).unwrap();
let astro_audio = chatter.astro_audio("r2d2").unwrap();
```

**AudioData 结构**

```rust
pub struct AudioData {
    pub samples: Vec<i16>,   // 原始采样数据 (i16)
    pub sample_rate: u32,    // 采样率 (如 44100)
    pub channels: u16,       // 声道数 (1 = 单声道, 2 = 立体声)
}
```

**cpal 播放示例**

```rust
use cpal::traits::{DeviceTrait, StreamTrait};

let audio = chatter.bd1_audio("hello", Mood::Happy).unwrap();

let device = cpal::default_output_device().unwrap();
let config = cpal::StreamConfig {
    channels: audio.channels,
    sample_rate: cpal::SampleRate(audio.sample_rate),
    buffer_size: cpal::BufferSize::Default,
};

let channels = audio.channels;
let samples = audio.samples.clone();

let stream = device.build_output_stream(
    &config,
    move |output: &mut [i16], _: &cpal::OutputCallbackInfo| {
        let frames = output.len() / channels as usize;
        for i in 0..frames {
            for c in 0..channels as usize {
                output[i * channels as usize + c] = samples[i + c];
            }
        }
    },
    |err| eprintln!("Error: {}", err),
    None,
)?;

stream.play()?;
std::thread::sleep(std::time::Duration::from_secs(2));
```

### 获取可用音效列表

```rust
use droid_chatter::{get_available_phrases, DroidType};

let phrases = get_available_phrases("sounds", DroidType::Do);
for p in phrases {
    println!("{}", p);
}
```

## 项目结构

```
droid-chatter/
├── src/
│   └── lib.rs       # 核心库
├── examples/
│   └── demo.rs      # 使用示例
└── sounds/          # 运行时自动下载（通过 setup_sounds）
```

## 情绪（Mood）说明

| 情绪 | 说明 |
|------|------|
| `Mood::Happy` | 开心、兴奋 |
| `Mood::Sad` | 悲伤、失落 |
| `Mood::Angry` | 愤怒、不满 |

## 运行示例

```bash
# 运行 demo（首次会自动下载音效）
cargo run --example demo

# 运行测试
cargo test
```

## 依赖

- `rodio` - 音频播放（基于 cpal）
- `hound` - WAV 文件读写
- `walkdir` - 目录遍历
- `thiserror` - 错误处理
- `reqwest` - 从 npm 下载音效
- `tar` / `flate2` - 解压 tgz 包

## 许可

ISC
