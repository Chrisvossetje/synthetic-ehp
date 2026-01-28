# Curtis Table to Algebraic EHP data

This python program constructs the Algebraic EHP sequence, both constructing every $H^*(\Lambda_n)$ and all differentials in the AEHP sequence. This generates the file algebraic_data.ts which is automatically placed inside /site/src. This python program is nothing special, it is mostly a formatting problem. 

It can be seen that the generators AEHP provide an upper bound to the generators (as $\Z[\tau]$-modules) of the SEHP. Or stated differently SEHP generators inject to AEHP generators (also in a compatible way, compatible meaning wrt. the AEHP filtration + Adams filtration).

The input to this is a (reduced) curtis table. This is a compact table giving you full information on all $H^*(\Lambda_n)$ and the differentials in the AEHP. We have copied the curtis table from William Balderrama's website[https://williamb.info/lambda/classic-curtis-table.txt]. Originally Martin C. Tangora has calculated this table in the paper "Computing the homology of the lambda algebra", appearing in the Memoirs of the American Mathematical Society Volume 58 Number 337 (1985). The original algorthim and name are due to Edward Curtis.