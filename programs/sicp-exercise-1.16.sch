(define (fast-expt-recursive b n)
  (cond ((= n 0) 1)
        ((even? n) (square (fast-expt-recursive b (/ n 2))))
        (else (* b (fast-expt-recursive b (- n 1))))
  )
)

(define (square n) (* n n))

(define (even? n) (= (remainder n 2) 0))

(test-eq (fast-expt-recursive 2 0) 1)
(test-eq (fast-expt-recursive 2 1) 2)
(test-eq (fast-expt-recursive 2 2) 4)
(test-eq (fast-expt-recursive 2 3) 8)
(test-eq (fast-expt-recursive 2 4) 16)
(test-eq (fast-expt-recursive 2 5) 32)
(test-eq (fast-expt-recursive 2 6) 64)

(define (fast-expt-iterative-helper b n a)
  (cond
    ((= n 0) a)
    ((even? n) (fast-expt-iterative-helper (* b b) (/ n 2) a))  ; figuring this out was an a-ha moment!
    (else (fast-expt-iterative-helper b (- n 1) (* a b)))
  )
)

(define (fast-expt-iterative b n)
  (fast-expt-iterative-helper b n 1)
)

(test-eq (fast-expt-iterative 2 0) 1)
(test-eq (fast-expt-iterative 2 1) 2)
(test-eq (fast-expt-iterative 2 2) 4)
(test-eq (fast-expt-iterative 2 3) 8)
(test-eq (fast-expt-iterative 2 4) 16)
(test-eq (fast-expt-iterative 2 5) 32)
(test-eq (fast-expt-iterative 2 6) 64)
