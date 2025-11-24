
![](https://pbs.twimg.com/media/G6f0DIEbcAAJFWD?format=jpg&name=medium)

# NeoFold

NeoFoldはNeoの実装で、コンパクトな実装かつ、端末での証明に最適化されるよう目指します。

Neoは、Wilson NguyenとSrinath Settyによって提案されたLatticeFoldの改良で、LatticeFoldの提案者のDan Boneh,Binyi Chenらの改良、LatticeFold+とは異なる進化と遂げています。
特に、Neoが提案するMatrix Commitment Schemeは、witnessの値の大きさが小さければ小さいほど、そのwitnessへのコミットのコストは下がります。ゼロに対してのコミットメントコストは完全にゼロになります。
これは、PedersenHashなどのDiscrete-log-basedのコミットメントが備えていた性質と似ており、NebulaなどのFoldingSchemeの研究によって培われてきたテクニックをそのまま使うことができます。

さらに、オリジナルの論文 LatticeFold で議論されていたAjtaiCommitmentをそのまま引き継いでいるため、Neoも同様に量子計算機耐性を有します。
見方を変えれば、NeoのMatrix Commitment Schemeは、LatticeFoldのRing上で定義されたAjtaiCommitmentの効率的な実装の一つと言えます。

LatticeFoldはNethermindによる部分的な実装がある一方で、Neoにはそれがないため、私たちがNeoの論文を元にNeoFoldを実装しました。

## 実装の方針

ark-worksのMLEは、evaluation formで多項式を管理しているのに対して、私たちの実装は、シンプルに多項式をRustのクロージャーとして定義します。
こうすることで、プログラムとしての式と、多項式としての式の見た目がほどんど一致し、サムチェックなどもRust的で直感的に記述することができます。

また、NeoではVerifier回路でもMLEを使用することも重要です。

回路システムについて、私たちは独自に作成したWasekiという回路システムを利用します。なぜなら、NeoFoldやzkVMそのものの実装で書かれる必要のある回路は、Rustで記述されるProverプログラムと密結合しており、回路部分に記述しやすいNoirなどの言語を使用することはできず、Rustの回路システムのデファクトスタンダードのark-r1cs-stdは、回路を極端に読みづらくする可能性がありました。
また、Nebulaで提案されているSwitchboardを記述するのが難しいかったのもあります。将来的にはWasekiの内部でark-r1cs-stdを使用し、互換のある回路を生成できることを目指しています。

現在、Wasekiは回路の書きやすさに極端に振っており、回路の生成に重大なパフォーマスの低下を及ぼす可能性があります。その極端な使用とは、回路の変数はCopyトレイトを実装しており、回路の制約やwitnessはプログラムのグローバルステートとして保存されます。これによって、回路記述者は、ark-ffのFieldの変数と同じように回路を書くことができます。この過激なデザインによるメリットは、ぜひNeoFoldの実装をご覧ください。デメリットは、先述の通り、回路の生成が遅くなる可能性があります。ただし、FoldinsgScheme全般に言えることですが、回路の生成は全体を通して何度も行うものではなく、本番環境では生成済みの回路を使い回すことができるので、プロダクションにはこの影響は及ぶことはありません。
VHDLのようなものだと考えてください。

Wasekiのデザインによって、回路とそれ以外のコードが密結合していて、どれが制約でどれがそうではないかを見分けづらいことがあります。
NeoFoldのコードを読む時は、型がVar<F>かFかをよく確認してください。Var<F>を使って行われる計算は全て制約式で、それ以外はwitnessを計算するためのproverのコードです。

例えば、次のコードは、回路の変数の64itのRangeCheckを行うコードです。
変数varの内部の値、valを取り出し、それをビット分解した値をwitnessとしてallocateし、それを2^iの定数による線型結合の結果が元の回路変数と一致するかを制約しています。

```rust
let var = Var::from(10);
let val = var.value().into_bigint();
(0..64)
    .map(|i| {
        let b = Var::from(val.get_bit(i));
        (2 << i) * b
    })
    .sum()
    .equal(var);
```



# Matrix Commitment Scheme

