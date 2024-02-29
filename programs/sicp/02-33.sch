; Defined in SICP 2.2.3
(define (accumulate op initial sequence)
  (if (null? sequence) initial
      (op (car sequence)
          (accumulate op initial (cdr sequence))
      )
  )
)

(define (map p sequence)
  (accumulate (lambda (x y) (cons (p x) y)) '() sequence)
)

(define (square x) (* x x))

(test-repr (map square '(1 2 3)) '(1 4 9))

(define (append seq1 seq2)
  (accumulate cons seq2 seq1)
)

(test-repr (append '(1 2) '(3 4)) '(1 2 3 4))

(define (length sequence)
  (accumulate (lambda (x y) (+ 1 y)) 0 sequence)
)

(test-repr (length '(1 2 3 4 5)) 5)
