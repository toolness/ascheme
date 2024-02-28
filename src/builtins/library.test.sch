(test-repr
  (filter (lambda (x) (not (null? x))) '(1 () 2 () 3))
  '(1 2 3)
)

(test-repr (reverse '(1)) '(1))
(test-repr (reverse '(1 2)) '(2 1))
(test-repr (reverse '(1 4 9 16 25)) '(25 16 9 4 1))
