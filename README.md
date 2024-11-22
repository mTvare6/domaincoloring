# tinydc

Phase portrait and domain coloring <300 loc rust.
![csc(1/z)](examples/a.png)

![sin((z-1)/z)](examples/b.png)

![sin(z^2 - 1)(z-(2+i))/(z^2 + (2+i))](examples/c.png)

# Building

```
cargo build --release
```

# Running

To change function modify manually. Check [`num::complex::Complex`](https://autumnai.github.io/cuticula/num/complex/struct.Complex.html)

```
cargo run --release
feh image.png
```

# TODO
- [ ] Add custom colormap as hex params
- [ ] Custom symbolic function as parameter
- [ ] Supersampling and scaling
