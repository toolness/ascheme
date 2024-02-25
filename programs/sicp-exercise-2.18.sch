(define (reverse x)
  (define (reverse-helper x result)
    (if (null? x)
      result
      (reverse-helper (cdr x) (cons (car x) result))
    )
  )
  (reverse-helper x '())
)

(print-and-eval (reverse (list 1)))
(print-and-eval (reverse (list 1 2)))
(print-and-eval (reverse (list 1 4 9 16 25)))
