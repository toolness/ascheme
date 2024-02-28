(define (fringe x)
  (if (pair? x)
    (if (null? (cdr x))
          (fringe (car x))
        (append (fringe (car x)) (fringe (cdr x)))
    )
    (list x)
  )
)

(define x (list (list 1 2) (list 3 4)))

(test-repr (fringe x) '(1 2 3 4))

(test-repr (fringe (list x x)) '(1 2 3 4 1 2 3 4))
