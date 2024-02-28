(test-repr
  (filter (lambda (x) (not (null? x))) '(1 () 2 () 3))
  '(1 2 3)
)

(test-repr (reverse '(1)) '(1))
(test-repr (reverse '(1 2)) '(2 1))
(test-repr (reverse '(1 4 9 16 25)) '(25 16 9 4 1))
; From R5RS 6.3.2
(test-repr (reverse '(a b c)) '(c b a))
(test-repr (reverse '(a (b c) d (e (f)))) '((e (f)) d (b c) a))

; From R5RS 6.3.2
(test-repr (append '(x) '(y)) '(x y))
(test-repr (append '(a) '(b c d)) '(a b c d))
(test-repr (append '(a (b)) '((c))) '(a (b) (c)))
(test-repr (append '() 'a) 'a)
(test-repr (append '(a b) '(c . d)) '(a b c . d))

(test-repr (append '(a b) '(c d) '(e f)) '(a b c d e f))
