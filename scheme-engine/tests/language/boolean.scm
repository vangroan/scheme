;; ========
;; Booleans
;; ========

(assert (boolean? #t))
(assert (boolean? #f))
(assert (not (boolean? 42)))

(assert (not (and 1 2 3 #f 5 6)))
