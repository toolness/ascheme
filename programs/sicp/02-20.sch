(define (same-parity? a b) (eq? (remainder a 2) (remainder b 2)))

(test-eq (same-parity? 5 2) #f)
(test-eq (same-parity? 4 2) #t)

; This is linear recursive, it could be linear iterative.
(define (filter x predicate)
  (cond ((null? x) '())
        ((predicate (car x)) (cons (car x) (filter (cdr x) predicate)))
        (else (filter (cdr x) predicate))
  )
)

(define (same-parity n . numbers)
  (cons n (filter numbers (lambda (x) (same-parity? n x))))
)

(test-repr (same-parity 1 2 3 4 5 6 7) '(1 3 5 7))
(test-repr (same-parity 2 3 4 5 6 7) '(2 4 6))
