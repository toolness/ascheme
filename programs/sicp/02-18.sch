(define (reverse x)
  (define (reverse-helper x result)
    (if (null? x)
      result
      (reverse-helper (cdr x) (cons (car x) result))
    )
  )
  (reverse-helper x '())
)

(test-repr (reverse '(1)) '(1))
(test-repr (reverse '(1 2)) '(2 1))
(test-repr (reverse '(1 4 9 16 25)) '(25 16 9 4 1))
