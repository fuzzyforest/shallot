(list
  (define 'defun (μ (name params value) (list define (list '' name) (list λ params value))))
  'code_below
  (defun funky_add (x y z) (+ x y z))
  (funky_add 1 2 3)

  (define π 3.14159)
  (defun circle_area (r) (* π r r))
  (define π 4)
  (circle_area 1)
  )
