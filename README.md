
# Matrix Commitment Scheme

Matrix Commitment Scheme は、Ajtaiコミットメントをベースにしつつ、NTTを削減し、それに伴いpay-per-bitによるコスト削減が実現されている。

Commitmentは、d*κのFのmatrixで、FはAlmost Golidlockという  (2^64 − 2^32 + 1) − 32 の素数を使ったfieldを使用する。

このコミットメントのセキュリティの議論は、Ajtaiコミットメントに準拠していて、定義上はRq上のMatrixの乗算として行われるが、実際にはcolumn-combinationのような行列の掛け算として処理される。

コミットメントされるwitnessは、Fqのsize mの vectorなのだが、これを拡張して、d*mのZqのmatrixとして定義される。そのmatrixのcolumn vectorは、Zqをビット分解した時の0,1になる。

このd*mのMatrixであるZに対してAjtaiコミットメントをするには、ZのcolumnベクトルをRqの係数とみなして、m次元のRqのベクトルといて、先ほど作ったd*κのFのmatrixとかける。

なので、その結果は、κのRqのベクトルなのだが、Rqはd次元のcolumnベクトルとして見れるので、コミットメントの形はd*κのZqのmatrixとして処理できる。
つまり、形式的にはRqの乗算になるのだが、実はこれはrot(a)*cf(b) = cf(a*b)という性質と、cf(b)はビットベクトルだという性質を使うと、cf(b)のそれぞれを係数とした、rot(a)のcolumnベクトルの加算として見れる。

さらに、cf(x)は、rot(a)に対して準同型だということは、rot(a)をコミットメントのランダム線型結合のチャレンジとして使うことができるということ。
今の設定だと、[-1,0,1,2]を係数にもつRqのrotation-matrixがチャレンジということになる。
ちなみに、κ=13, d=64, m=2^26になる。


Commitmentのやり方。
z: Vec<F> が回路のwitnessで、これをVec<[F; D]>にする。[F;D]はFをビット分解したD個の値の列。
A: Vec<Vec<Rq>>がAjtaiのMatrixなのだが、これは、Vec<Vec<[F;D]>>として定義する。この時の[F;D]はRqの係数の列。
このA*Zの行列の掛け算を行う。ただし、要素そのものの掛け算は、rot(a)*cf(b)を行う。

