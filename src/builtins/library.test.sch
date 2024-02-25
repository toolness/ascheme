(test-repr
  (filter (lambda (x) (not (null? x))) '(1 () 2 () 3))
  '(1 2 3)
)
