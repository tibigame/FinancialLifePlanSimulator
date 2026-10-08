# FinancialLifePlanSimulator

日本向けの金融・ライフプランシミュレーターです。

## 必要なライブラリ

- Rust（MSVC toolchain）
- Visual Studio Build Tools（C++ によるデスクトップ開発）
- Node.js と Bun（Bun がない場合は Yarn）
- WebView2 Runtime（アプリの実行に必要）

## Windows でのビルド

リポジトリのルートで実行します。

```bat
build_debug.bat
build_release.bat
```

各スクリプトがフロントエンドとアプリをビルドし、実行ファイルをルートに出力します。
