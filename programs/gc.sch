(define x (quote (1 . 2)))

; this creates a cycle
(set-cdr! x x)

(define y (quote (3 . 4)))

; This creates a closure.
(define (add-n n) (lambda (x) (+ x n)))
(define add-one (add-n 1))

(gc)
(stats)

; Make the object involved in the cycle unreachable.
(define x 0)

(stats)
(gc)
(stats)
