(define (f-recursive n)
  (if (< n 3)
      n
      (+ (f-recursive (- n 1))
         (* 2 (f-recursive (- n 2)))
         (* 3 (f-recursive (- n 3)))
      )
  )
)

(define (f-iterative-helper a b c count) 
  (if (= 0 count)
      a
      (f-iterative-helper
        b             ; b is the new a
        c             ; c is the new b
        (+ c          ; the new c is based on the previous state
           (* 2 b)
           (* 3 a))
        (- count 1)
      )
  )
)

(define (f-iterative n)
  (f-iterative-helper 0 1 2 n)
)

(test-eq (f-recursive 0) 0)
(test-eq (f-recursive 1) 1)
(test-eq (f-recursive 2) 2)
(test-eq (f-recursive 3) 4)

(test-eq (f-iterative 0) 0)
(test-eq (f-iterative 1) 1)
(test-eq (f-iterative 2) 2)
(test-eq (f-iterative 3) 4)
(test-eq (f-iterative 15) (f-recursive 15))
