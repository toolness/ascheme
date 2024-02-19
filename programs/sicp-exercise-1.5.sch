(define (p) (p))

(define (test x y)
  (if (= x 0)
      0
      y))

(test-eq (test 1 2) 2)
(test-eq (test 0 2) 0)

(display "About to infinitely loop because we use") (newline)
(display "applicative-order evaluation and 'p' is tail recursive.") (newline)

(test 0 (p))
