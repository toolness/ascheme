(define (abs x)
  (if (< x 0)
    (- x)
    x
  )
)

(define (newline) (display "\n"))

(define (zero? x) (= x 0))

(define (null? x) (eq? x '()))

; Interestingly, `filter` isn't in R5RS, but it *is* in R6RS.
(define (filter predicate x)
  ; TODO: This is linear recursive, it could be linear iterative.
  (cond ((null? x) '())
        ((predicate (car x)) (cons (car x) (filter predicate (cdr x))))
        (else (filter predicate (cdr x)))
  )
)

(define (reverse x)
  (define (reverse-helper x result)
    (if (null? x)
      result
      (reverse-helper (cdr x) (cons (car x) result))
    )
  )
  (reverse-helper x '())
)

; TODO: Make this linear iterative.
(define (append a . more-lists)
  (define (append-two a b)
    (if (null? a) b
        (cons (car a) (append-two (cdr a) b))
    )
  )
  (if (null? more-lists) a
      (apply append (cons (append-two a (car more-lists)) (cdr more-lists))))
)
