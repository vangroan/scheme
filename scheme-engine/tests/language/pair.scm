;; ========
;; Pairs
;; ========

(define a (cons 1 2))
(assert (pair? a))
(assert (not (pair? 1)))
