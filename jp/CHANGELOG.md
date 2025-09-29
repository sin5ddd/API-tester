# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project adheres to Semantic Versioning.

## [v0.1.0] - 2025-09-29
### Added
- ネイティブ GUI（eframe/egui）による REST API テスターを初公開。
- HTTP メソッド: GET / POST / PUT / PATCH / DELETE。
- 背景スレッド送信（UI フリーズ防止）。
- 認証: None / Basic / Bearer / OAuth2 Token。
- ヘッダ編集（`Key: Value` 形式）。
- Body エディタ:
  - JSON 入力（整形表示）。
  - Form（key=value）エディタ（有効/無効、削除、+Add）。
  - JSON と Form の双方向同期（常に一致）。
- レスポンス表示:
  - ツリー表示（スクロール対応）。
  - ステータス色分け（200=緑、3xx/その他=黄、4xx/5xx=赤）。
- 差分:
  - 構造的 diff（`json_patch::diff`）。
  - インライン差分ハイライト（add/replace/remove など op 別カラー）。
  - 置換ノードの旧値をグレーで併記。
- クエリ（常時ツリーへ適用）:
  - JSONPath（抽出）。
  - JMESPath（抽出＋整形）。
- 検索（テキスト）: 整形 JSON 内の該当行を列挙。
- プロファイル管理（プロジェクト/フォルダ単位保存）:
  - 保存場所: `./profiles/<project>/<profile>.json`。
  - フォルダ風エクスプローラ UI（新規/保存/上書き/読み込み/削除/リネーム）。
- cURL インポート（method/url/headers/body の簡易解析）。
- エクスポート: JSON、NDJSON（配列）、CSV（配列のフラット化）。
- 自動ポーリング（間隔指定）。
- ウィンドウの「常に手前に表示」（Float/Unfloat）。
- コンパクト UI（行間・余白・フォントを詰めて高密度表示）。

### CI/Release
- Git タグ（例: `v0.1.0`）の push をトリガーに、GitHub Actions が Windows/Linux/macOS 向けにリリースビルドを作成。
- 各 OS 向けアーカイブ（tar.gz）と SHA256 チェックサムを自動生成。
- Release へアセットを自動添付。

### Notes
- Windows で起動しない場合は Visual C++ 再頒布可能パッケージが必要な場合があります。
- 巨大配列の仮想スクロールは未対応（今後の改善項目）。
