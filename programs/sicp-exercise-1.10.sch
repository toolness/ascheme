(define (A x y)
  ;(print-and-eval x)
  ;(print-and-eval y)
  (cond ((= y 0) 0)
        ((= x 0) (* 2 y))
        ((= y 1) 2)
        (else (A (- x 1) (A x (- y 1))))
  )
)

(print-and-eval (A 1 10))
(print-and-eval (A 2 4))
(print-and-eval (A 3 3))

(define (f n) (A 0 n))      ; Computes 2n
(define (g n) (A 1 n))      ; Computes 2^n
(define (h n) (A 2 n))      ; Order is 2, 4, 16, 65536. It's the previous entry squared.
