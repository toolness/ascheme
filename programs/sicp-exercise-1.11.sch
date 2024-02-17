(define (f-recursive n)
  (if (< n 3)
      n
      (+ (f-recursive (- n 1))
         (* 2 (f-recursive (- n 2)))
         (* 3 (f-recursive (- n 3)))
      )
  )
)

(test-eq (f-recursive 0) 0)
(test-eq (f-recursive 1) 1)
(test-eq (f-recursive 2) 2)
(test-eq (f-recursive 3) 4)
