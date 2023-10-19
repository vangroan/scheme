
;; =======
;; Lambdas
;; =======

(define add-self
  (lambda (x) (+ x x)))
(display (add-self 7)) (newline)
(assert (= (add-self 7) 14))

(define add-self (lambda (x) (+ x x))) (assert (= (add-self 7) 14))