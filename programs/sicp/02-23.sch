(define (first . lists)
  (cond ((null? lists) '())
        (else (cons (car (car lists)) (apply first (cdr lists))))
  )
)

(define (pop-first . lists)
  (cond ((null? lists) '())
        ((null? (cdr (car lists))) '())
        (else (cons (cdr (car lists)) (apply pop-first (cdr lists))))
  )
)

(define (zip . lists)
  (if (null? lists) '()
    (cons (apply first lists) (apply zip (apply pop-first lists)))
  )
)

(test-repr (first) '())
(test-repr (first '(1)) '(1))
(test-repr (first '(1) '(2)) '(1 2))
(test-repr (first '(1 2) '(3 4)) '(1 3))

(test-repr (pop-first '(1 2) '(3 4)) '((2) (4)))
(test-repr (pop-first '(1) '(3)) '())

(test-repr (zip '(1 2) '(3 4)) '((1 3) (2 4)))

(define (for-each-helper predicate list-of-args)
    (if (null? list-of-args) #!void
      (begin
        (apply predicate (car list-of-args))
        (for-each-helper predicate (cdr list-of-args))
      )
    )
)

; Note that supporting variadic args isn't actually part of 2.23,
; but the "real" for-each supports it, so I wanted to too.
(define (for-each predicate . lists)
  (for-each-helper predicate (apply zip lists))
)

(display "SICP exercise 2.23 output:") (newline)

(for-each (lambda (x) (newline)
            (display x))
          (list 57 321 88))

(newline) (newline)
(display "my output:") (newline)

(for-each (lambda (x y) (newline)
            (display (+ x y)))
          '(1 2 3) '(4 5 6))
