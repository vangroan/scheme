
;; ============
;; Conditionals
;; ============

;; if

(assert (= (if #f 1 2) 2))
(assert (= (if #t 1 2) 1))
(assert (if 0 #t #f))
(assert (if 1 #t #f))
(assert (if 3 #t #f))
(assert (= (if #t (+ 3 7) (* 3 7)) 10))
(assert (= (if #f (+ 3 7) (* 3 7)) 21))
(assert (if (= 3 3 3 3 3 3) #t #f))
;; (assert-eq (if (< 1 0)) #!void)

(define iter (lambda (n)
  (if (< n 6)
    (iter (+ n 1)) ; recursive
    (* n 3)        ; base
   )))
(assert-eq (iter 0) 18)

;; cond
(assert-eq 'first  (cond (#t 'first) (#t 'second) (#f 'third)) )
(assert-eq 'second (cond (#f 'first) (#t 'second) (#t 'third)) )
(assert-eq 'third  (cond (#f 'first) (#f 'second) (#t 'third)) )
;; (assert-eq (cond (#f 1) (#f 2) (#f 3)) #!void) ;; TODO: void literal
(assert-eq 'first  (cond ((< 0 1) 'first) ((> 0 1) 'second)) )
(assert-eq 'second (cond ((> 0 1) 'first) ((< 0 1) 'second)) )
(assert-eq 'second (cond (#f 'first) (else 'second)) )

(define choose (lambda (n) (cond ((< n 0) 1) ((= n 0) 2) ((> n 0) 3))))
(assert (= (choose 0) 2))
(assert (= (choose 1) 3))

(define iter (lambda (n)
  (cond
    ((>= n 5) (* n n))           ; base
     (else (iter (+ n 1)))       ; recursive
  )))
(assert-eq (iter 5) 25)
