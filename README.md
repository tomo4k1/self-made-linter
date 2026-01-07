<div align="center">

# 🚀 Speedy Nuxt Linter

[![Rust](https://img.shields.io/badge/built_with-Rust-dca282.svg)](https://www.rust-lang.org/)
[![Nuxt](https://img.shields.io/badge/Nuxt-00C58E?logo=nuxt.js&logoColor=white)](https://nuxt.com/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Nuxt.js プロジェクトのための、爆速・激かわリンターだよ〜✨**
<br>
Rust製だからマジで速いし、設定もラクラク！

</div>

---

## 📖 目次 (Table of Contents)

- [✨ 特徴 (Features)](#-特徴-features)
- [📦 インストール (Installation)](#-インストール-installation)
- [🚀 使い方 (Usage)](#-使い方-usage)
- [⚙️ 設定 (Configuration)](#-設定-configuration)
- [🛠️ 開発 (Development)](#-開発-development)

---

## ✨ 特徴 (Features)

*   **⚡️ 爆速パフォーマンス**: Rust で書いてるから、ファイルが多くても一瞬で終わるよ！
*   **🛡️ .gitignore 対応**: `.gitignore` を勝手に読み込んで、不要なファイルはスキップするよ。賢い〜！
*   **🔧 設定ファイル対応**: `.linterrc.json` でルールの ON/OFF ができるよ。プロジェクトに合わせてカスタマイズしてね💕
*   **📊 JSON出力**: CI/CD にも組み込みやすい JSON 出力モード搭載！

---

## 📦 インストール (Installation)

まだパッケージマネージャには登録してないから、ソースからビルドしてね🙏

```bash
# リポジトリをクローン
git clone https://github.com/tomo4k1/self-made-linter.git
cd self-made-linter

# ビルド（Releaseモード推奨✨）
cargo build --release
```

---

## 🚀 使い方 (Usage)

ビルドしたバイナリを実行するだけ！

```bash
# カレントディレクトリをチェック
./target/release/linter-test .

# 特定のディレクトリをチェック
./target/release/linter-test ./components

# JSON形式で出力（CI連携とかに便利！）
./target/release/linter-test . --json
```

---

## ⚙️ 設定 (Configuration)

ルートディレクトリに `.linterrc.json` を置いてね。
ルールごとに `"off"`, `"warn"`, `"error"` が選べるよ（現状は `"off"` かそれ以外かで判定してるけどね😅）。

**Example `.linterrc.json`:**

```json
{
  "rules": {
    "no-console": "off",
    "no-v-html": "error",
    "vue/mustache-interpolation-spacing": "error"
  }
}
```

### 📏 Supported Rules

| Rule Name | Description | Default |
| :--- | :--- | :--- |
| `no-console` | `console.log` とかの使用を禁止するよ🙅‍♀️ | `error` |
| `no-process-env` | `process.env` はセキュリティ的に危ないからダメ！ | `error` |
| `no-v-html` | XSSの危険がある `v-html` は使わないで！ | `error` |
| `vue/require-v-for-key` | `v-for` には `:key` が必須だよ🔑 | `error` |
| `vue/mustache-interpolation-spacing` | `{{ value }}` のスペースはちゃんと空けてね✨ | `error` |
| `nuxt/prefer-import-meta` | `process.env` より `import.meta.env` を使おう！ | `error` |

---

## 🛠️ 開発 (Development)

機能追加とかバグ修正とか、プルリク待ってるよ〜💖

```bash
# テスト実行
cargo test

# 開発用実行
cargo run -- fixtures/phase6/valid.vue
```

---

<div align="center">

Made with 💖 by Gal Engineer

</div>
