# LISP

function(a, b, c)
a + b
1 + 2 * 3

double = lambda x: 2*x

def do_twice(f, x):
   returns f(f(x))

double(17)
do_twice(double, 31)

(function a b c)
(+ a b)
(+ 1 (* 2 3))

(define double (lambda (x) (* 2 x)))
(defun double (x) (* 2 x))

(lambda (f x) (f (f x)))

# Why another interpreter

## Embeddable

## Extensible

### Provide functions
### Provide types

## Goal / limitation

enum Expression {
  void* ...,
  type_id ...
}

HTTP -> URI -> String -> Stdlib -> Core
     -> Rexeg -> String -> Core

Shallot




