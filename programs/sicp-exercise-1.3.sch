; SICP Exercise 1.3

(define (sum-of-squares a b) (+ (* a a) (* b b)))
(define (smallest? a b c) (and (< a b) (< a c)))
(define (sum-of-biggest-squares a b c)
  (cond ((smallest? a b c) (sum-of-squares b c))
        ((smallest? b a c) (sum-of-squares a c))
        (else (sum-of-squares a b))))

(test-eq (< 0 1) #t)
(test-eq 5 5)
(test-eq (sum-of-squares 2 3) 13)
(test-eq (smallest? 1 2 3) #t)
(test-eq (smallest? 5 2 3) #f)
(test-eq (sum-of-biggest-squares 1 2 4) 20)
(test-eq (sum-of-biggest-squares 2 1 4) 20)
(test-eq (sum-of-biggest-squares 4 1 2) 20)
(test-eq (sum-of-biggest-squares 4 0 2) 20)
