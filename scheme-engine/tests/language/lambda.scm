
;; =======
;; Lambdas
;; =======

(define add-self
  (lambda (x) (+ x x)))
(display (add-self 7)) (newline)
(assert (= (add-self 7) 14))

(define add-self (lambda (x) (+ x x)) (assert (= (add-self 7) 14)))

;; Test nested lambda calls
(define add-add-self (lambda (a b) (+ (add-self a) (add-self b)))) (assert (= (add-add-self 7 11) 36))
