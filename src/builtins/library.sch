(define (abs x)
  (if (< x 0)
    (- x)
    x
  )
)

(define (newline) (display "\n"))

(define (zero? x) (= x 0))

(define (null? x) (eq? x '()))
