# Bibliography: Swarm Geometry Foundations

This bibliography documents the foundational publications and original papers underpinning the Swarm Geometry concepts, optimization frameworks, optimal transport metrics, and topological data analysis tools described in the thesis.

---

## 1. Hahn-Banach Separation Theorem

### Hans Hahn (1927)
*   **Title:** Über lineare Gleichungssysteme in linearen Räumen
*   **Author:** Hans Hahn
*   **Year:** 1927
*   **Publication Venue:** *Journal für die reine und angewandte Mathematik* (Crelle's Journal), Vol. 157, pp. 214–229.
*   **DOI:** [10.1515/crll.1927.157.214](https://doi.org/10.1515/crll.1927.157.214)
*   **URL:** [https://doi.org/10.1515/crll.1927.157.214](https://doi.org/10.1515/crll.1927.157.214)
*   **BibTeX:**
    ```bibtex
    @article{hahn1927lineare,
      author  = {Hahn, Hans},
      title   = {{\"U}ber lineare Gleichungssysteme in linearen R{\"a}umen},
      journal = {Journal f{\"u}r die reine und angewandte Mathematik},
      volume  = {157},
      pages   = {214--229},
      year    = {1927},
      doi     = {10.1515/crll.1927.157.214},
      url     = {https://doi.org/10.1515/crll.1927.157.214}
    }
    ```

### Stefan Banach (1929)
*   **Title:** Sur les fonctionnelles linéaires
*   **Author:** Stefan Banach
*   **Year:** 1929
*   **Publication Venue:** *Studia Mathematica*, Vol. 1, No. 1, pp. 211–216.
*   **DOI:** [10.4064/sm-1-1-211-216](https://doi.org/10.4064/sm-1-1-211-216)
*   **URL:** [http://eudml.org/doc/216976](http://eudml.org/doc/216976)
*   **BibTeX:**
    ```bibtex
    @article{banach1929fonctionnelles,
      author  = {Banach, Stefan},
      title   = {Sur les fonctionnelles lin{\'e}aires},
      journal = {Studia Mathematica},
      volume  = {1},
      number  = {1},
      pages   = {211--216},
      year    = {1929},
      doi     = {10.4064/sm-1-1-211-216},
      url     = {http://eudml.org/doc/216976}
    }
    ```
*   **Swarm Relevance:** Provides the mathematical foundation for separating convex sets in infinite-dimensional vector spaces. In Swarm Geometry, the separation theorem guarantees the existence of separating hyperplanes that define boundaries between distinct agent behaviors, control regimes, and obstacle-avoidance domains.

---

## 2. Farkas' Lemma & Optimization Extensions

### Julius Farkas (1902)
*   **Title:** Über die Theorie der einfachen Ungleichungen
*   **Author:** Julius (Gyula) Farkas
*   **Year:** 1902
*   **Publication Venue:** *Journal für die reine und angewandte Mathematik* (Crelle's Journal), Vol. 124, pp. 1–27.
*   **DOI:** [10.1515/crll.1902.124.1](https://doi.org/10.1515/crll.1902.124.1)
*   **URL:** [http://www.digizeitschriften.de/main/dms/img/?PPN=GDZPPN002165023](http://www.digizeitschriften.de/main/dms/img/?PPN=GDZPPN002165023)
*   **BibTeX:**
    ```bibtex
    @article{farkas1902theorie,
      author  = {Farkas, Julius},
      title   = {{\"U}ber die Theorie der einfachen Ungleichungen},
      journal = {Journal f{\"u}r die reine und angewandte Mathematik},
      volume  = {124},
      pages   = {1--27},
      year    = {1902},
      doi     = {10.1515/crll.1902.124.1},
      url     = {http://www.digizeitschriften.de/main/dms/img/?PPN=GDZPPN002165023}
    }
    ```

### Harold W. Kuhn & Albert W. Tucker (1951)
*   **Title:** Nonlinear Programming
*   **Authors:** Harold W. Kuhn and Albert W. Tucker
*   **Year:** 1951
*   **Publication Venue:** *Proceedings of the Second Berkeley Symposium on Mathematical Statistics and Probability*, University of California Press, Berkeley, CA, pp. 481–492.
*   **DOI:** [10.1525/9780520313330-026](https://doi.org/10.1525/9780520313330-026)
*   **URL:** [https://projecteuclid.org/euclid.bsmsp/1200500249](https://projecteuclid.org/euclid.bsmsp/1200500249)
*   **BibTeX:**
    ```bibtex
    @inproceedings{kuhntucker1951nonlinear,
      author    = {Kuhn, Harold W. and Tucker, Albert W.},
      title     = {Nonlinear Programming},
      booktitle = {Proceedings of the Second Berkeley Symposium on Mathematical Statistics and Probability},
      pages     = {481--492},
      publisher = {University of California Press},
      address   = {Berkeley, California},
      year      = {1951},
      doi       = {10.1525/9780520313330-026},
      url       = {https://projecteuclid.org/euclid.bsmsp/1200500249}
    }
    ```
*   **Swarm Relevance:** Farkas' Lemma acts as a fundamental theorem of the alternative for systems of linear inequalities. Extended by the Karush-Kuhn-Tucker (KKT) conditions, it establishes necessary and sufficient conditions for constrained optimal control, permitting decentralized optimization and coordination feasibility guarantees across the agent collective.

---

## 3. Wasserstein Space & Optimal Transport Geodesics

### Gaspard Monge (1781)
*   **Title:** Mémoire sur la théorie des déblais et des remblais
*   **Author:** Gaspard Monge
*   **Year:** 1781
*   **Publication Venue:** *Histoire de l'Académie Royale des Sciences de Paris, avec les Mémoires de Mathématique et de Physique pour la même année*, pp. 666–704.
*   **URL:** [https://gallica.bnf.fr/ark:/12148/bpt6k35800/f796](https://gallica.bnf.fr/ark:/12148/bpt6k35800/f796)
*   **BibTeX:**
    ```bibtex
    @article{monge1781memoire,
      author  = {Monge, Gaspard},
      title   = {M{\'e}moire sur la th{\'e}orie des d{\'e}blais et des remblais},
      journal = {Histoire de l'Acad{\'e}mie Royale des Sciences de Paris, avec les M{\'e}moires de Math{\'e}matique et de Physique pour la m{\^e}me ann{\'e}e},
      pages   = {666--704},
      year    = {1781},
      url     = {https://gallica.bnf.fr/ark:/12148/bpt6k35800/f796}
    }
    ```

### Leonid V. Kantorovich (1942)
*   **Title:** On the Translocation of Masses
*   **Author:** Leonid V. Kantorovich
*   **Year:** 1942
*   **Publication Venue:** *Comptes Rendus (Doklady) de l'Académie des Sciences de l'URSS*, Vol. 37, No. 7-8, pp. 227–229 (Original Russian title: *О переносе масс*, published in *Doklady Akademii Nauk SSSR*).
*   **DOI:** [10.1287/mnsc.5.1.1](https://doi.org/10.1287/mnsc.5.1.1) *(Note: DOI belongs to the English translation published in Management Science, Vol. 5, No. 1, pp. 1-4, 1958)*
*   **URL:** [http://web.eecs.umich.edu/~pettie/matching/Kantorovitch-translocation-of-masses-1942.pdf](http://web.eecs.umich.edu/~pettie/matching/Kantorovitch-translocation-of-masses-1942.pdf) *(Original English reprint)*
*   **BibTeX:**
    ```bibtex
    @article{kantorovich1942translocation,
      author  = {Kantorovich, Leonid V.},
      title   = {On the Translocation of Masses},
      journal = {Doklady Akademii Nauk SSSR},
      volume  = {37},
      number  = {7--8},
      pages   = {227--229},
      year    = {1942},
      note    = {English translation in Management Science, Vol. 5, No. 1, pp. 1--4, 1958, DOI: 10.1287/mnsc.5.1.1}
    }
    ```

### Leonid N. Vaserstein (1969)
*   **Title:** Markov processes over denumerable products of spaces, describing large systems of automata
*   **Author:** Leonid N. Vaserstein (spelled "Wasserstein" in Western literature)
*   **Year:** 1969
*   **Publication Venue:** *Problemy Peredachi Informatsii* (Problems of Information Transmission), Vol. 5, No. 3, pp. 64–72.
*   **URL:** [http://mi.mathnet.ru/ppi1811](http://mi.mathnet.ru/ppi1811)
*   **BibTeX:**
    ```bibtex
    @article{vaserstein1969markov,
      author  = {Vaserstein, Leonid N.},
      title   = {Markov processes over denumerable products of spaces, describing large systems of automata},
      journal = {Problemy Peredachi Informatsii},
      volume  = {5},
      number  = {3},
      pages   = {64--72},
      year    = {1969},
      url     = {http://mi.mathnet.ru/ppi1811},
      note    = {English translation in Problems of Information Transmission, Vol. 5, No. 3, pp. 47--52}
    }
    ```
*   **Swarm Relevance:** The Monge-Kantorovich formulation of optimal transport, combined with Vaserstein's metric, provides a rigorous distance metric (Wasserstein distance) on probability measures. It allows tracking, measuring, and interpolating (geodesics) the density distribution transitions of agent swarms to achieve global reconfigurations with minimal displacement.

---

## 4. Persistent Homology

### Herbert Edelsbrunner, David Letscher, & Afra Zomorodian (2000)
*   **Title:** Topological Persistence and Simplification
*   **Authors:** Herbert Edelsbrunner, David Letscher, and Afra Zomorodian
*   **Year:** 2000
*   **Publication Venue:** *Proceedings of the 41st Annual IEEE Symposium on Foundations of Computer Science (FOCS 2000)*, pp. 454–463.
*   **DOI:** [10.1109/SFCS.2000.892133](https://doi.org/10.1109/SFCS.2000.892133)
*   **URL:** [https://doi.org/10.1109/SFCS.2000.892133](https://doi.org/10.1109/SFCS.2000.892133)
*   **Journal Version Reference:**
    *   **Venue:** *Discrete & Computational Geometry*, Vol. 28, No. 4, pp. 511–533, 2002
    *   **DOI:** [10.1007/s00454-002-2885-2](https://doi.org/10.1007/s00454-002-2885-2)
*   **BibTeX:**
    ```bibtex
    @inproceedings{edelsbrunner2000topological,
      author    = {Edelsbrunner, Herbert and Letscher, David and Zomorodian, Afra},
      title     = {Topological Persistence and Simplification},
      booktitle = {Proceedings of the 41st Annual IEEE Symposium on Foundations of Computer Science},
      pages     = {454--463},
      year      = {2000},
      doi       = {10.1109/SFCS.2000.892133},
      url       = {https://doi.org/10.1109/SFCS.2000.892133}
    }
    ```

### Afra Zomorodian & Gunnar Carlsson (2005)
*   **Title:** Computing Persistent Homology
*   **Authors:** Afra Zomorodian and Gunnar Carlsson
*   **Year:** 2005
*   **Publication Venue:** *Discrete & Computational Geometry*, Vol. 33, No. 2, pp. 249–274.
*   **DOI:** [10.1007/s00454-004-1146-y](https://doi.org/10.1007/s00454-004-1146-y)
*   **URL:** [https://doi.org/10.1007/s00454-004-1146-y](https://doi.org/10.1007/s00454-004-1146-y)
*   **BibTeX:**
    ```bibtex
    @article{zomorodian2005computing,
      author  = {Zomorodian, Afra and Carlsson, Gunnar},
      title   = {Computing Persistent Homology},
      journal = {Discrete \& Computational Geometry},
      volume  = {33},
      number  = {2},
      pages   = {249--274},
      year    = {2005},
      doi     = {10.1007/s00454-004-1146-y},
      url     = {https://doi.org/10.1007/s00454-004-1146-y}
    }
    ```
*   **Swarm Relevance:** Persistent Homology introduces an algebraic framework for computing topological features (e.g., connected components, loops, voids) across multiple spatial scales. In Swarm Geometry, it enables the characterization of formation shape structures, ensuring the collective tracks global topological properties and detects spatial voids/obstacles in noisy environments.