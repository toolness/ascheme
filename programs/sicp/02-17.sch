(define (last-pair x)
  (if (null? (cdr x))
    x
    (last-pair (cdr x))
  )
)

(test-repr (last-pair (list 23 72 149 34)) '(34))
