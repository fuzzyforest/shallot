(list
  (define 'defun (μ (name params value) (list define (list '' name) (list λ params value))))
  (defun target (x) (- (* x x) 2))
  (define 'ε 0.00001)
  (defun ∂ (f x) (/ (- (f (+ x ε)) (f (- x ε))) (* 2 ε)))
  (defun newton-improve (f x) (- x (/ (f x) ((∂ f) x))))

  (defun abs (x) (cond (≤ x 0) (- 0 x) x))

  (define 'solve (λ (f self x)
    (cond (≤ (abs (f x)) ε)
      x
      (self f (newton-improve f x))
    )
  ))

  (defun Y (f) (
       (λ (x) (f (λ (y) ((x x) y))))
       (λ (x) (f (λ (y) ((x x) y))))
  ))

  (define 'factorial (λ (factorial) (λ (x)
    (cond (= 0 x)
      1
      (* x (factorial (- x 1)))
    )
  )))

  (print ((Y factorial) 5))
  (print ((Y (solve target)) 1))
 )
