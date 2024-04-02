# mytaskbar

## 何を目的としたプログラムか
　Win10までタスクバーにあった２つの機能は、Win11では使えなくなってしまいました。

* 「ツールバー」
* 「ラベル表示されたウィンドウリストからウィンドウを切り替え」

後者は23H2でラベル表示は復活しましたが、タスクバー表示位置を左右に振ることが依然できないため、ウィンドウの数が多いと「ラベルを一望してのウィンドウ切替」ということができない状況です。

Win11において、Win10タスクバーを復活する手順はありますが、安心して長らく使っていくには不安があります。これらをWin11で代替する目的で、Win10タスクバーにあった「ツールバー」や「ラベル付きウィンドウの縦並びリスト」をホットキーで画面の任意位置、任意タイミングで表示して利用できるプログラムです。

■機能
　①アプリランチャー（アイコン表示、ファイル一覧表示）
　②ウィンドウ切替（ウィンドウ一覧表示）

■画面
　①アプリランチャー（アイコン表示、ファイル一覧表示）
　　①-A)子フォルダ表示プロパティ（フォルダ単位）
　②ウィンドウ切替（ウィンドウ一覧表示）
　③ホットキー設定
　④ウィンドウリスト:ソート設定

■インストールとアンインストール
　「mytaskbar.exe」を任意のフォルダに配置します。exe単体で動作し、付属するファイルはありません。
　迷う人はとりあえず、デスクトップに置いて起動しても大丈夫です。
　このプログラムは設定をレジストリに保存しますが、後からexeの配置を変えても設定に影響しません。

　アンインストールは、exeファイルとレジストリ「HKEY_CURRENT_USER\SOFTWARE\myprogram」を削除します。
　レジストリの意味が分からない人は放っておいても問題にはなりません。

■プログラムの起動と終了
　exeを起動するとウィンドウの表示無く常駐し、タスクバー通知アイコンやホットキーを通して操作します。
　タスクバー通知アイコンは、マウスカーソルをあてて「mytaskbar」と表示されるアイコンが対象です。
　プログラムの終了方法は、タスクバー通知アイコン「mytaskbar」を右クリックし「終了」を選択します。

■mytaskbar.exeの起動引数オプション

　exeの引数は、後述する二重起動時のホットキー動作指定(半角2文字)だけです。
　それ以外にオプションはありません。

■タスクバー通知アイコンの変更方法


　　～～～～～～～


■アプリランチャーの使い方（はじめの一歩）

　基本的な考え方は、Win10までにあったタスクバーのツールバー（昔はクイック起動と呼ばれていた機能）と全く同じです。
　一般的なランチャーのように、どんなアプリを起動するかをランチャー用に設定するような使い方ではありません。

　大まかな流れ
　①任意の専用フォルダを作って、そこによく使うアプリやファイルのショートカットを集めます。
　②ホットキー設定画面で登録します。
　③設定したホットキーで①のフォルダをランチャー表示します。

　詳細な手順
　①-1)フォルダ作成
　　任意の場所にフォルダを作成します。デスクトップ上でも構いません。後で好きな位置に変えることもできます。
　　ここでは、デスクトップ上に「taskbar」というフォルダを作成したと仮定します。

　①-2)ショートカット配置
　　「taskbar」フォルダ内にランチャー表示したいアプリのショートカットを配置します。
　　「taskbar」をデスクトップ上でダブルクリックしてエクスプローラを表示しておきます。
　　そして、スタートメニューにあるアプリなら、そのアプリを「taskbar」へドラッグすることでショートカットを配置できます。
　　スタートメニューになくても任意のexeやbatファイルなどについて、エクスプローラ上で右クリックしてショートカットを作成し、
　　「taskbar」へ配置することができます。
　　アプリである必要もなく、よく使うフォルダへのショートカットや、よく使うwordやexeclファイル等へのショートカットでも構いません。
　　ショートカットである必要もないのでexeやbatファイルの実体やwordファイルの実体等、どんなファイルを置いても構いません。

　②)ホットキー設定画面
　　上で作った「taskbar」フォルダをアイコンランチャーとして表示してみましょう。
　　「mytaskbar.exe」を起動すると、タスクバー右側の通知アイコン欄に「mytaskbar」のアイコンが追加されます。
　　アイコンを右クリックしてポップアップメニューから「ホットキー設定」を選んでください。

　　設定ダイアログが表示されたら、「追加」ボタンをクリックしてホットキーを一つ追加します。
　　以下のように入力してください。

　　・ランチャーを選択(デフォルトのまま)。
　　・SHIFT, ALT, NONEを選択欄から選び、好きなキー(a-z,1-0,記号)を１文字入力。
　　　NONEはここでは選ばない(使い方は後述)。CTRL+SHIFT(ALT)+入力した文字がホットキーとなる。
　　　例として、SHIFT,Aを入力したとします。

　　・ターゲット：上で作成した「taskbar」フォルダのフルパスを入力
　　　　C:\Users\～あなたのユーザ名～\Desktop\taskbar
　　・表示：アイコンを選択
　　・アイコンサイズ：大(デフォルトのまま)
　　・ウィンドウサイズ：W「0」×H「0」(デフォルトのまま)
　　・システムファイル：「表示」チェックを外す(デフォルトのまま)

　　・右下「設定」ボタンを押して、ダイアログを閉じます。

　③)ホットキーを押す
　　ダイアログを閉じたら、入力したホットキーを押します。
　　例ではCTRL+SHIFT+Aキーを押します。
　　「taskbar」フォルダに配置したショートカットのアイコンが並んで表示されます。
　　好きなアイコンをクリックし起動することを確かめてください。
　　アイテムをミドルボタンでドラッグするとソート順を変えることができます。

　ここまでが、アプリランチャーの基本的な使い方です。設定項目の詳細や応用例は後述します。

■ウィンドウ切替の使い方（はじめの一歩）

　大まかな流れ
　①ホットキー設定画面で登録します。
　②設定したホットキーでウィンドウ一覧を表示します。

　詳細な手順
　①)ホットキー設定画面
　　「mytaskbar.exe」を起動すると、タスクバー右側の通知アイコン欄に「mytaskbar」のアイコンが追加されます。
　　アプリランチャーの手順を実行済みで、すでに「mytaskbar.exe」を起動している場合は二重起動は不要です。
　　通知アイコンを右クリックしてポップアップメニューから「ホットキー設定」を選んでください。

　　設定ダイアログが表示されたら、「追加」ボタンをクリックしてホットキーを一つ追加します。
　　以下のように入力してください。

　　・「ランチャー」欄を「ウィンドウリスト」に選択変更。
　　・SHIFT, ALT, NONEを選択欄から選び、好きなキー(a-z,1-0,記号)を１文字入力。
　　　NONEはここでは選ばない(使い方は後述)。CTRL+SHIFT(ALT)+入力した文字がホットキーとなる。
　　　例として、SHIFT,Zを入力したとします。

　　・ウィンドウサイズ：W「0」×H「0」(デフォルトのまま)
　　・その他の項目は無効化され変更不可

　　・右下「設定」ボタンを押して、ダイアログを閉じます。

　②)ホットキーを押す
　　ダイアログを閉じたら、入力したホットキーを押します。
　　例ではCTRL+SHIFT+Zキーを押します。
　　いま開いているウィンドウのアイコンとラベルの一覧が縦で表示され、左クリックでウィンドウを切り替えることができます。
　　右クリックでシステムメニューを開くことができます(閉じる、最大化、最小化など)。
　　一覧は同じアプリでグループ化され、左ドラッグにてグループ単位のソート順を入れ替えることができます。

　　ここまでが、ウィンドウ切替の基本的な使い方です。アイテムのソートに関しての設定方法は後述します。

■既知の事象
　◎ウィンドウ切替のソート順はタスクバーと連動しません。
　◎アプリランチャーでzipをフォルダ扱いでファイル一覧表示が可能ですが、クリックでの操作はできません。

■トラブルシューティング

　アプリランチャー／ウィンドウ切替共通
　　ホットキーを押しても表示されない
　　　◎別アプリとホットキーが衝突していないか：
　　　　別アプリをすべて停止させてみる、ホットキーの指定を変えてみる
　　　◎ホットキー設定画面のウィンドウサイズ指定が小さすぎる：
　　　　W、Hともにゼロを指定してみる

　　二重起動でランチャーが表示できない
　　　◎引数が間違っていないか：
　　　　引数は半角大文字で2文字。受け付けるのはS?、A?、N?のみ。?の部分は1-0、A-Z、-^|@[;:],./_、のいずれか

　アプリランチャー
　　ホットキーを押しても表示されない
　　　◎ターゲットの指定が間違っていないか：
　　　　　ターゲットの指定を空白にして、デスクトップが表示されるか確かめる
　　　　　→表示される場合は、ターゲット指定が間違っている可能性が高い。
　　　　　対象のフォルダをエクスプローラ等でSHIFT+右クリックし、「パスのコピー」を選んで、ターゲット指定に張り付けてみる

　・ウィンドウ切替
　　ホットキーを押しても表示されない
　　　◎表示に気づきにくい：
　　　　　別のアプリが一つも起動していないと、一覧ウィンドウがとても小さくなる。別アプリを起動してみる。

■mytaskbar.exeの二重起動

　・初回にexe起動すると常駐し、２回目以降は新たなプログラムは起動せず、既に常駐しているプログラムに作用します。
　　以下のホットキー設定をしないまま、２回目起動すると、エラーダイアログが表示されます。

　・ホットキー設定画面にて、キー指定を「NONE」＋「!」とすると、２回目起動時の動作を指定することができます。

　応用例：タスクバーのボタンでウィンドウ切替の一覧を表示

　　・エクスプローラ上でmytaskbar.exeを右クリックして、「タスクバーにピン留めする」を選ぶ。
　　・ホットキー設定画面で「ウィンドウリスト」、「NONE」＋「!」を設定。
　　・使い方のイメージ）
　　　ＰＣを起動したらまずタスクバーのピン留めボタンで本プログラムの１回目を起動。
　　　２回目以降、クリックするとウィンドウ切替一覧が表示される。
　　・ホットキーではなく、タスクバーに備わった機能であるかのような操作感でウィンドウ切替を利用可能。

　応用例：タスクバーのピン留めアイコンを変えたい

　　このプログラムの使い方ではありませんが・・・。
　　・mytaskbar.exeを右クリックして仮初めのショートカットを作成する。
　　・ショートカットを右クリックしてプロパティを開き、「アイコンの変更」ボタンをクリックして、アイコンを設定。
　　・プロパティを閉じ、ショートカットを右クリックして「タスクバーにピン留めする」を選ぶ。
　　・仮初めのショートカットファイルは消しても問題なし。

　・exe起動時に引数を与えると、ホットキーの動作を押したのと同じ動作をさせることができます。
　　引数には例の通り、2文字指定します。半角指定で、アルファベットは大文字しか反応しません。

　　　CTRL+SHIFT+Aの設定を動作させたい場合　→　mytaskbar.exe SA
　　　CTRL+ALT+Dの設定を起動させたい場合　→　mytaskbar.exe AD

　・ホットキー設定画面における「NONE」の意味は、ホットキーを登録せずにexe二重起動だけ使いたい時に設定します。
　　特に「!」は特殊な意味として、引数無しの時の挙動を指定します。
　　「NONE」にもキーとして「1～0、A～Z、-^|@[;:],./_」の48種(!を入れると49種)が指定可能です。
　　ホットキーのように実際に押すわけではないので、キーに何を指定するかは引数を区別する意味しかありません。
　　「mytaskbar.exe」(引数なし)、「mytaskbar.exe N!」は同じ意味で、どちらも同じ動作をします。

　　　NONE+Aの設定を動作させたい場合　→　mytaskbar.exe NA

　応用例：タスクバーのボタンでアプリランチャーを表示

　　上記の引数なしの二重起動によるウィンドウ切替の一覧のピン留めを生かしつつ、もうひとつ、
　　アプリランチャーのピン留めを追加します。

　　・ホットキー設定画面でアプリランチャーを設定します。ホットキーを使うならお好みのキーを指定します。
　　　ホットキーを使わない場合は、「NONE」＋「1」などを設定します。ここでは「SHIFT」＋「A」
　　・mytaskbar.exeを右クリックして仮初めのショートカットを作成する。
　　・ショートカットを右クリックしてプロパティを開き、リンク先の末尾に空白＋「SA」を追加（SAの部分は例の場合）。
　　・プロパティを閉じ、ショートカットを右クリックして「タスクバーにピン留めする」を選ぶ。
　　・仮初めのショートカットファイルは消しても問題なし。
　　・これで、アプリランチャーもタスクバーに備わった機能であるかのような操作感で利用可能。

　応用例：バッチファイルがタスクバーのピン留めできないんだけど

　　もはやこのプログラムとは何の関係もありませんが・・・一応。
　　・cmd.exeへのショートカットを作成し、引数として、「/c "バッチファイルのフルパス"」とすればピン留め可能。

■ホットキー設定画面の詳細

　ここまででホットキー設定画面のうち、キー指定欄の説明をしました。残りの部分を説明します。

　・右下の「設定」ボタンは設定を反映し、ダイアログを閉じます。「キャンセル」は設定を破棄してダイアログを閉じます。

　・キー指定欄で「ウィンドウリスト」を設定している時
　　「ウィンドウサイズ」の項目
　　　ピクセル単位でのW：幅、H:高さの一覧ウィンドウの表示領域を制限できます。
　　　「0」指定は画面領域が許す限りウィンドウが広がります（マルチディスプレイの場合、またぐことはありません）。
　　　幅が表示必要量よりも小さくなったときは、ウィンドウタイトルの中間部分が「...」表示となります。
　　　高さが表示必要量よりも小さくなったときは、一覧ウィンドウがスクロール表示になります。
　　　ホイールスクロール（ミドルボタンのコロコロ）か、一覧ウィンドウ下部の矢印でスクロールできます。

　・キー指定欄で「ランチャー」を設定している時

　　ターゲット：表示したいフォルダを設定する。空白はデスクトップを意味する。
　　表示：「リスト」か「アイコン」を選ぶ。
　　アイコンサイズ：「アイコン」の大小を選ぶ。
　　ウィンドウサイズ：
　　　表示が「リスト」か「アイコン」かで意味が変わる。
　　　「リスト」→W,Hの意味は「ウィンドウリスト」の時と同じく、ピクセル単位で表示領域を制限する。
　　　「アイコン」→W,Hの意味はピクセル単位ではなく、アイコンの数を表す。
　　　　W,Hとも「0」指定→なるべく正方形(平方数)に近い形で、端数は横長になるようにアイコンをレイアウトする。
　　　　W,H片方だけ「0」指定→非ゼロ指定した幅or高さを守る形でレイアウトする。
　　　　W,H両方を非ゼロ指定→幅and高さを守る形でレイアウトする。
　　　いずれもあふれる場合はスクロール表示となる。
　　　スクロールは上下のため、高さのみ指定は端数スクロールが頻発し非推奨。幅とともに指定したほうが良い。
　　システムファイル：「表示」にチェックを入れるとdesktop.iniなどのファイルもランチャーに表示します。

■アプリランチャー：操作と設定の詳細

　アプリランチャーには、リスト表示とアイコン表示がありますが、表示の違いだけで持っている機能や操作は全く同じです。

　左クリック：アイテムの標準動作（基本は開く）／子フォルダ実体の子ウィンドウ表示
　左ダブルクリック：（子フォルダ実体のみ）アイテムの標準動作（Windows標準状態ではエクスプローラを開く）
　左ドラッグ：アイテムのドラッグ＆ドロップ
　ミドルドラッグ：アイテムのソート変更
　右クリック：アイテムのポップアップメニューを表示
　右ドラッグ：アイテムのドラッグ＆ドロップ（ドロップ時のポップアップメニュー表あり）

　ランチャー余白：ランチャー下部の余白（スクロール時は矢印の部分）や、アイコン表示の端数で生じた余白の部分
　ランチャー余白への右クリック：ランチャー用フォルダ自身のポップアップメニューを表示
　ランチャー余白へのドロップ：ランチャー用フォルダ自身へのアイテムコピー・移動・リンク操作

　これまでの説明の通り、アプリランチャーは特定のフォルダを指定して表示します。
　フォルダの中に子フォルダ(実体)が含まれる場合は、それをクリックすると階層をたどる形で小ウィンドウが表示されます。
　親のアイコン／リスト表示の如何にかかわらず、子フォルダの表示はデフォルトでリスト表示となります（後述の設定で変更可能）。
　フォルダ階層が深い場合でも、順番にクリックしていくことで目的のファイルにたどり着くことができます。
　また、子フォルダ実体はダブルクリックでエクスプローラを表示できます。

　一方でフォルダへのショートカットをクリックしても小ウィンドウは表示されません。Windows標準状態ではエクスプローラが表示されます。
　ランチャーとして使う場合、子フォルダ実体なのかショートカットなのかで挙動が大きく違うので注意してください。

　応用例：フォルダ実体のアイコン表示を変える

　　フォルダ実体のアイコンはWindowsシェルの機能で変更可能です。フォルダを右クリックして「プロパティ」を開き、「カスタマイズ」タブから
　　「アイコンの変更」ボタンを押すことで、アイコンを選択して変更することができます。

　応用例：Nethood、シンボリックリンク

　　アプリランチャーのファイル列挙にはWindowsシェル機能を使っているので、デスクトップ上のゴミ箱も見えるし、
　　Nethood機能やファイル/フォルダのシンボリックリンクも問題なく機能します(Nethoodやシンボリックリンクは各自で調べてください)。
　　ランチャー用の専用フォルダに子フォルダの実体を切っていく使い方以外にも
　　シンボリックリンクを張って子フォルダ実体のように子ウィンドウで階層をたどるような使い方も可能です。

　応用例：簡易エクスプローラ

　　ホットキー設定画面にて、アプリランチャーをターゲットを空白（デスクトップ）にして、リスト表示指定しておくと、
　　たどる階層が深くなるかもしれませんが、PC内外のどのファイルにもたどり着くことができます。
　　現在のウィンドウを最小化したりエクスプローラをわざわざ開くまでもない、ちょっとしたファイル操作をランチャー機能で対応できます。
　　Win10までタスクバーのツールバーを使っていれば、同様の利用をしていた人も多いのではないかと思います。


　ランチャー内の各アイテムは右クリックでエクスプローラ相当のポップアップメニューを表示し操作できます。
　ランチャー下部の余白もしくはスクロール矢印の部分、もしくはアイコン表示の端数で生じた余白の部分（以下「ランチャー余白」）は、
　ランチャー用に作ったフォルダ自身（ホットキー設定画面で設定したターゲットフォルダ）のポップアップメニューを表示します。
　ここでランチャー用に作ったフォルダ自身を名前変更したり削除さえできますが、当然ながらそれ以降ランチャー表示はできなくなります。

　アイテムを左クリックのドラッグすることでドラッグアンドドロップ(D&D)が可能です。ランチャーはD&Dを受けることもできます。
　アイテム上へドロップするとそのアイテムのアプリを開く操作となり、「ランチャー余白」へのドロップは、
　ランチャー用に作ったフォルダ自身（ホットキー設定画面で設定したターゲットフォルダ）へのファイルコピー・移動・リンク操作です。

　応用例：ランチャーへのアイコン追加

　　ランチャー用に作ったフォルダ自身へのD&D操作が可能であることを利用すると、
　　ランチャーへのアイコン追加のためにエクスプローラを経由する必要はありません。

　　・初回のフォルダ作成だけはエクスプローラで済ませ、ホットキー設定画面にてアプリランチャーを設定します。
　　・スタートメニューから追加したいアプリをドラッグを開始する。
　　・左ドラッグしたまま、ホットキーでアプリランチャーを表示する。
　　・アプリランチャーウィンドウの「ランチャー余白」部分へドロップ。
　　・アプリランチャーを表示しなおすと、アプリアイコンが追加されていることが確認できる。

　応用例：ウェブサイトの簡易ブックマーク共有

　　ブックマーク用フォルダを作り、ランチャー登録をしておきます。
　　edge、chrome、firefoxなどブラウザのURL欄の左側ボタンをドラックするとURLショートカットを作成できるので
　　上と同じ操作でランチャーウィンドウへD&Dして、ブックマークサイトを増やしていけます（リスト表示がいいでしょう）。
　　ブックマーク用フォルダをOneDriveやBoxDriveの上に置くと、ブラウザや端末を超えてブックマーク共有できます。

　応用例：ランチャーを使ったD&Dアプリ起動

　　あるファイルを普段はメモ帳でいいのだけど、たまにはサクラエディタで開きたいというケースへの対応事例です。
　　エクスプローラのシェル拡張で右クリックメニューにサクラエディタを登録する以外にも、以下のような応用ができます。

　　・アプリランチャーにサクラエディタを登録しておく（ランチャーはアイコン表示の方が便利でしょう）。
　　・デスクトップやエクスプローラ上で、開きたいファイルを左ドラッグ開始。
　　・左ドラッグしたまま、ホットキーでアプリランチャーを表示。
　　・アプリランチャー上のサクラエディタのアイコンへドロップ。

　ランチャーウィンドウの「ランチャー余白」部分を右クリックしたときに、ポップアップメニューの最上部に
　「子フォルダ表示プロパティ」が表示されます。これはそのフォルダに対する、ランチャー表示方法を設定することができます。
　子フォルダ実体を子ウィンドウで表示するときはデフォルトでリスト表示ですが、これを変更できます。設定項目の内容はホットキー画面と同じです。
　注意点は、ここでの表示設定はランチャーで子フォルダをたどって開く時だけに有効で、
　ホットキーで初回表示する時の表示方法はホットキー設定画面で設定する必要があります。

　例で説明します。
　デスクトップに「taskbar」フォルダを配置し、ホットキー設定画面でアイコン表示設定をしたと仮定します。
　設定どおりのホットキーで「taskbar」フォルダを表示したときはアイコン表示になります。

　別に、デスクトップをリスト表示するホットキーを設定したと仮定します。この時、デスクトップ配下にある「taskbar」を子フォルダを
　たどる形で表示することができますが、この時の「taskbar」フォルダはアイコンではなくリスト表示になります。

　このように、親フォルダからたどった時もホットキーと同じようにアイコン表示にしたいという場合に
　「taskbar」フォルダを表示しているランチャーウィンドウの「ランチャー余白」部分を右クリックし「子フォルダ表示プロパティ」で設定変更します。

　この「子フォルダ表示プロパティ」は絶対パスで保持するため、たとえばシェル最上位のデスクトップからたどったらアイコン表示だけど、
　C:\users\username\desktopからたどったらリスト表示、ということはできません。
　一方で同じランチャー用のフォルダに対してホットキー設定画面から複数のホットキーを設定することはできます。
　ある組み合わせのホットキーではアイコン表示だけど、別のホットキーではリスト表示、ということが可能です。


　最後に、ランチャーアイテムのソートについてです。
　ランチャーウィンドウでアイテムをミドルクリックでドラッグすると、ソート順を変えることができます。
　スクロールをまたがなければならないときは、ミドルドラッグしたままホイールスクロールするか（ちょっと難易度高い）、
　ミドルドラッグしたまま下部の矢印にホバー（カーソルをあててじっとしておく）するとスクロールさせることが可能です。

　ソート順は絶対パスで保持し、ホットキー／子フォルダ表示で共用です。ホットキー用の個別ソート設定はありませんので、
　ホットキーの組み合わせを変えてアイコン表示とリスト表示を併用した場合でも、ソート順を分けることはできません。
　ランチャーウィンドウの「ランチャー余白」部分を右クリックし「ソート順リセット」を選ぶと、Windowsシェルの名前順に戻ります。

■ウィンドウ切替：操作と設定の詳細

　ウィンドウ切替で各アイテムに対し可能な操作は左クリック：ウィンドウ切替、右クリック：システムメニューを開く、の2点だけです。
　この項での説明事項は、すべてアイテムのグルーピングやソート順に関する仕様や操作に関する説明のみです。

　左クリック：ウィンドウを切り替える
　右クリック：ウィンドウのシステムメニューを表示する(閉じる、最大化、最小化など)

　左ドラッグ：グループのソート順変更
　ミドルドラッグ：１つのウィンドウのソート変更
　ミドルクリック：１つのウィンドウのグループ変更

　・アイテムのグルーピング

　　ウィンドウ切替一覧上で、左側にグループを見分けるバー表示があります。

　　　　自動グループ：単一メンバでバー表示なし／複数メンバで黒いバー表示
　　　　任意グループ：単一・複数メンバとも薄緑バー表示

　　ウィンドウ切替一覧における各アイテムは、プロセスイメージ名※が同一の時に自動でグルーピングします。
　　自動グループには、プロセスイメージ名が異なるアイテムが所属することはできません。
　　　※プロセスイメージ名：ウィンドウを表示しているプロセスのexeファイルの絶対パス

　　その他に、利用者が任意にグループを指定できる任意グループがあります。
　　任意グループには、プロセスイメージ名が異なるアイテムも含め、利用者が所属アイテムを自由に設定できます。
　　（目的が同一のexcelとwordのウィンドウを同一グループにするなど）

　　新しいウィンドウができると、新しい自動グループができるか既存の自動グループのメンバが増えます。
　　一方で任意グループのメンバが自動で増えることはありません。

　・アイテムのソート
　　ウィンドウ切替一覧におけるソートはグループ単位で行います。（グループ化されていないアイテムも、メンバが一つのグループとみなす）。
　　このグループ単位のソートは左ドラッグで行います。

　　複数のメンバが存在するグループの場合、グループ内でのメンバのソートも変えることができます。
　　グループ内のメンバのソートはミドルクリックで行います。
　　アイテムのミドルドラッグでは、対象アイテムをグループまたぎで移動することもできます（後述の制約があります）。

　　ソートがスクロールをまたぐ場合は、ドラッグしたままホイールスクロールするか（ミドルボタンの時はちょっと難易度高い）、
　　ドラッグしたまま下部の矢印にホバー（カーソルをあててじっとしておく）するとスクロールさせることが可能です。

　・グルーピングの変更
　　自動グループにはプロセスイメージ名が異なるアイテムが所属できないので、自動グループしか無い初期状態ではグループ間の移動は全くできません。
　　グループに関するルールを書きますがちょっと複雑です。実際にミドルクリックとミドルドラッグをいろいろやってみてください。

　　ウィンドウ切替一覧における各アイテムに対して、ミドルクリックするとアイテムのグループの種類を変更することができます。

　　自動グループに属しているアイテムはグループから外れ、単一の任意グループのアイテムになります。
　　複数メンバの任意グループに属しているアイテムはグループを外れて、単一の任意グループのアイテムになります。
　　任意グループに対しては、任意のアイテムをミドルドラッグの操作にて所属させることができます。

　　単一の任意グループのアイテムに対して、ミドルクリックすると自動グループに戻すことができます。
　　ただし、同じプロセスイメージ名を持つ自動グループがないことが条件になります。
　　ほかに同じプロセスイメージ名を持つ自動グループがあるときは、任意グループのアイテムを自動グループに所属させることができます。

　・ソート順の事前設定

　　◎ウィンドウのソート順は揮発性です。プログラムが終了すると残りません。タスクバー上の並び順とも連動しません。
　　ただし、自動グループの並び順については、あらかじめ指定しておくことができます。

　　・並び順を設定したいウィンドウを一つ以上起動しておきます。
　　　起動していないウィンドウはソート設定画面に候補表示されないので、ソート設定できません。
　　・タスクバー通知アイコン「mytaskbar」を右クリックし「ウィンドウリスト:ソート設定」を選びます。
　　・起動しているプロセスイメージ名(exeファイル)の一覧が表示されます。
　　　アイテムをミドルクリックすると、アイコン左側に黒い四角が表示されます。
　　　黒いバーがついているアイテムはソート順の指定対象となります。
　　　ソート指定対象のアイテムは左ドラッグで順序を入れ替えることができます。
　　・「ＯＫ」ボタンで設定が反映されます。

　　◎プログラムを起動してから一度でもウィンドウ切替を表示していると、その時に表示されたソート順が保持されるので、
　　上記の設定画面で設定したソート順が反映されるわけではありません。
　　設定の効果を確認したい場合は、本プログラムを一度終了させて再起動し、ウィンドウ切替を表示してみてください。

　　設定画面で設定したアイテムは一覧の下の方に指定した順で並び、ソート対象指定しないプロセスは、一覧の上の方に
　　ランダムに並ぶことが確認できると思います。ソート指定されていない新しいウィンドウは上の方に並ぶ仕組みです。
　　（新ウィンドウを生成し、ホットキーでウィンドウ切替を表示し選ぶときに、新しいものが上にあってほしいという考え方）
　　自分が起動する可能性のあるプロセスをしらみつぶしにソート順設定をしておくと並びを固定化できます。

　　◎対象のプロセスイメージ(exeファイルのフルパス)がバージョンアップでどんどん名前が変わってしまうケースがあります。
　　パスが変わる都度、ソート設定を変える煩雑さの回避のため、ワイルドカードを指定することができます。
　　画面で一旦ソート設定したうえでレジストリ値を修正します。意味が分からない人は危険のため、操作しないでください。

　　・レジストリエディタを起動します。
　　・「HKEY_CURRENT_USER\SOFTWARE\myprogram\mytaskbar」キーを開きます。
　　・「win_sort」値を開きます。
　　・該当のフルパスのうち、一部をアスタリスクに置き換えます。
　　　C:\Program Files\WindowsApps\MSTeams_nnnnn.nnn.nnnn.nnn_x64__8wekyb3d8bbwe\ms-teams.exe
　　　　↓
　　　C:\Program Files\WindowsApps\MSTeams_*_x64__8wekyb3d8bbwe\ms-teams.exe
　　・アスタリスクは複数設定可能で、case sensitiveの単純な貪欲検索です。プロセスイメージが複数ヒットしてしまうような
　　　緩いワイルドカードでは動作がおかしくなるので注意してください。
