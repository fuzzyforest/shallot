def apply(f, x):
    return globals()[f](x)

def double(x):
    return x*x

apply(double, 7)
