(define (cons x y)
  (lambda (m) (m x y)))

(define (car z)
  (z (lambda (p q) p)))

(test-eq (car (cons 'a 2)) 'a)

(define (cdr z)
  (z (lambda (p q) q)))

(test-eq (cdr (cons 'a 2)) 2)
