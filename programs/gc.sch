(define x (quote (1 . 2)))
(define y (quote (3 . 4)))
(set-cdr! x x)
(define x 0)
(stats)
(gc)
(stats)
