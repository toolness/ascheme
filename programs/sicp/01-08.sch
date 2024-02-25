(define (cube-root-iter guess x)
  (if (good-enough? guess x)
    guess
    (cube-root-iter (improve guess x) x)
  )
)

(define (good-enough? guess x)
  (define delta (abs (- (cube guess) x)))
  (define delta-as-fraction-of-x (/ delta x))
  ;(print-and-eval x)
  ;(print-and-eval guess)
  ;(print-and-eval delta)
  ;(print-and-eval delta-as-fraction-of-x)
  (< delta-as-fraction-of-x 0.0001)
)

(define (square x) (* x x))

(define (cube x) (* x x x))

(define (improve guess x)
  (/ 
    (+
      (/ x (square guess))
      (* 2 guess)
    )
    3
  )
)

(define (cube-root x)
  (cube-root-iter 1.0 x))

(print-and-eval (cube-root 27))
