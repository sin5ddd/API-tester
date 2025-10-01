# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project adheres to Semantic Versioning.

## [v0.1.3] - 2025-10-01
### Fixed
- **リクエスト送信時にフォームフィールドの値の型が正しく保持されるように修正**
  - 以前は、Content-Type: application/json を使用した Form モードで、型セレクター（Float、Int、Bool）に関わらず、すべてのフィールド値が文字列に変換されていました
  - 現在は各フィールドの型セレクターが尊重されます:
    - Int フィールドは JSON 数値（整数）として送信されます
    - Float フィールドは JSON 数値（浮動小数点）として送信されます
    - Bool フィールドは JSON ブール値（true/false）として送信されます
    - String フィールドは JSON 文字列のまま送信されます
  - 無効な数値は文字列表現にグレースフルにフォールバックされます
  - この修正により、すべてが文字列ではなく、正しい型のデータを API が受信できるようになりました

## [v0.1.2] - 2025-09-30
### Added
- レスポンス受信時に JSON ツリーのルートノードが自動的に展開され、即座にアクセスしやすくなりました
- プロファイルがバイナリディレクトリではなく、ユーザーホームディレクトリに保存されるようになりました
    - Windows: `%USERPROFILE%\.api-tester\profiles\`
    - Mac/Linux: `~/.api-tester/profiles/`
- 自動マイグレーション: バイナリディレクトリに存在する既存のプロファイルは、初回起動時にホームディレクトリに自動的に移動されます
- Windows でコンソールウィンドウが非表示になりました（リリースビルドのみ。デバッグビルドではデバッグ用に表示されます）
- JSON 型保持機能を備えた型付きフォームフィールド
    - 6 つのデータ型をサポート: String、Int、Float、Bool、Array、Object
    - 各フィールドの型セレクタードロップダウン
    - 数値（Int/Float）は JSON でダブルクォートでエスケープされなくなりました
    - ブール値は JSON の true/false に適切に変換されます
    - Array と Object 型は再帰構造を持つネストされた子フィールドをサポートします
    - Array/Object 型にネストされたフィールドを追加するための「+ Add child」ボタン
    - ネスト構造の視覚的インデント（レベルごとに 20px）
    - JSON から Form モードに切り替える際の自動型検出
- シンタックスハイライト付き高度な JSON エディタ
  - JSON ボディ編集用に基本的な TextEdit を `egui_code_editor` に置き換えました
  - GRUVBOX カラーテーマによるシンタックスハイライト
  - 行番号表示
  - 12 行エディタ、フォントサイズ 14
  - ワンクリックで JSON を整形する「Format JSON」ボタン
- Headers セクションの Content-Type セレクター
  - 5 つの一般的な Content-Type オプションを持つ ComboBox ドロップダウン:
    - application/json（デフォルト）
    - application/x-www-form-urlencoded
    - text/plain
    - application/xml
    - multipart/form-data
  - 変更時にヘッダーテキストフィールドの Content-Type ヘッダーが自動的に更新されます

### Changed
- egui 0.27 から egui 0.32.3 への移行
    - eframe、egui、egui_extras の依存関係をバージョン 0.32.3 に更新しました
    - 新しい eframe API で要求される `Result` 型を返すようにアプリ作成コールバックを修正しました
    - 非推奨の `CollapsingHeader::id_source` を `id_salt` に置き換えました
    - コードの明確性を向上させるために反駁不可能なパターンマッチングをリファクタリングしました
    - すべての既存機能は保持されており、ユーザー機能に破壊的な変更はありません

- プロファイルパネルが自動縮小動作を使用するようになりました
  - 固定の max_height(220.0) から auto_shrink([false, true]) に変更しました
  - パネルはプロジェクトとプロファイルの数に応じて自動的に拡大/縮小します
  - 画面スペースの利用が改善されました
- Headers セクションがデフォルトで折りたたまれた状態で開始されるようになりました
  - `ui.collapsing()` から `CollapsingHeader::default_open(false)` に変更しました
  - 起動時の視覚的な混乱を軽減します

### Fixed
- **Form key=value モードが Headers の Content-Type 設定を尊重するようになりました**
  - 以前は、Form モードは Headers に関係なく常に `application/x-www-form-urlencoded` としてデータを送信していました
  - 現在は Headers の Content-Type をチェックし、それに応じて body をフォーマットします:
    - Headers に `Content-Type: application/json` が設定されている場合、フォームフィールドは JSON 形式に変換されます
    - それ以外の場合は、以前と同様に form-urlencoded 形式を使用します
  - すべての body モードで Headers 情報が適切に適用されるようになりました

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
