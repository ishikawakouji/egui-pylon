# egui を使って basler のGUI

まずは、egui の hello-world を参照

その次に、日本語フォントを利用できるように改造

pylon-cxx を利用すると basler に接続できたのでこれを利用できないかと開始。

## 難所
pylon-cxx のカメラのインスタンスは Pylon の参照を必要としているが
これを構造体の中でうまく初期化することに苦労した。

```Rust
pub struct InstantCamera<'a> {
    #[allow(dead_code)]
    lib: &'a Pylon,
    inner: cxx::UniquePtr<ffi::CInstantCamera>,
    #[cfg(feature = "stream")]
    fd: RefCell<Option<tokio::io::unix::AsyncFd<std::os::unix::io::RawFd>>>,
}
```

結局は、[こちら](https://medium.com/@reduls/refers-other-field-in-the-same-struct-in-rust-777bb2075b8c)を参考に `unsafe`を使ってみた。

```Rust
    let pylon = pylon_cxx::Pylon::new();
    let lefp: &'cam _ = unsafe { &*(&pylon as *const _) };
    let camera = pylon_cxx::TlFactory::instance(lefp)
        .create_first_device()
        .unwrap();

    Self { camera, pylon }
```

さらに、メモリ開放時におかしなことになるんじゃないかと予想しているんだけど、終了時にエラーになるので、構造体の定義も工夫。

```Rust
struct GrabApp<'cam> {
    // メモリを開放して欲しい順？にフィールドを記述
    camera: InstantCamera<'cam>,
    #[allow(dead_code)]
    pylon: Pylon,
}
```

## 残っていること
エミュレータで動作確認しているが、どこかからの転送が間に合ってないのか、画像がところどころ繋がっていない。

カメラ実物を使った確認。内部的に白黒画像であることを前提とした処理がある。