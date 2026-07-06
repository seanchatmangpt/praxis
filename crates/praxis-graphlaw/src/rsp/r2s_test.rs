#![cfg(test)]

use crate::rsp::r2s::Relation2StreamOperator;
    use crate::rsp::r2s::StreamOperator::{DSTREAM, ISTREAM, RSTREAM};
    use crate::sparql::Binding;

    #[test]
    fn test_rstream() {
        let new_result = vec![
            vec![
                Binding {
                    var: "?1".to_string(),
                    val: "1".to_string(),
                },
                Binding {
                    var: "?2".to_string(),
                    val: "2".to_string(),
                },
            ],
            vec![
                Binding {
                    var: "?1".to_string(),
                    val: "1.2".to_string(),
                },
                Binding {
                    var: "?2".to_string(),
                    val: "2.2".to_string(),
                },
            ],
        ];
        let mut s2r: Relation2StreamOperator<Vec<Binding>> =
            Relation2StreamOperator::new(RSTREAM, 0);
        let expected_result = new_result.clone();

        assert_eq!(expected_result, s2r.eval(new_result, 1));
    }
    #[test]
    fn test_dstream() {
        let old_result = vec![
            vec![
                Binding {
                    var: "?1".to_string(),
                    val: "1".to_string(),
                },
                Binding {
                    var: "?2".to_string(),
                    val: "2".to_string(),
                },
            ],
            vec![
                Binding {
                    var: "?1".to_string(),
                    val: "1.2".to_string(),
                },
                Binding {
                    var: "?2".to_string(),
                    val: "2.2".to_string(),
                },
            ],
        ];
        let new_result = vec![
            vec![
                Binding {
                    var: "?1".to_string(),
                    val: "1".to_string(),
                },
                Binding {
                    var: "?2".to_string(),
                    val: "2".to_string(),
                },
            ],
            vec![
                Binding {
                    var: "?1".to_string(),
                    val: "1.3".to_string(),
                },
                Binding {
                    var: "?2".to_string(),
                    val: "2.3".to_string(),
                },
            ],
        ];
        let expected_deletion = vec![vec![
            Binding {
                var: "?1".to_string(),
                val: "1.2".to_string(),
            },
            Binding {
                var: "?2".to_string(),
                val: "2.2".to_string(),
            },
        ]];
        let mut s2r: Relation2StreamOperator<Vec<Binding>> =
            Relation2StreamOperator::new(DSTREAM, 0);
        s2r.eval(old_result, 1);

        assert_eq!(expected_deletion, s2r.eval(new_result, 2));
    }
    #[test]
    fn test_istream() {
        let old_result = vec![
            vec![
                Binding {
                    var: "?1".to_string(),
                    val: "1".to_string(),
                },
                Binding {
                    var: "?2".to_string(),
                    val: "2".to_string(),
                },
            ],
            vec![
                Binding {
                    var: "?1".to_string(),
                    val: "1.2".to_string(),
                },
                Binding {
                    var: "?2".to_string(),
                    val: "2.2".to_string(),
                },
            ],
        ];
        let new_result = vec![
            vec![
                Binding {
                    var: "?1".to_string(),
                    val: "1".to_string(),
                },
                Binding {
                    var: "?2".to_string(),
                    val: "2".to_string(),
                },
            ],
            vec![
                Binding {
                    var: "?1".to_string(),
                    val: "1.3".to_string(),
                },
                Binding {
                    var: "?2".to_string(),
                    val: "2.3".to_string(),
                },
            ],
        ];
        let expected_deletion = vec![vec![
            Binding {
                var: "?1".to_string(),
                val: "1.3".to_string(),
            },
            Binding {
                var: "?2".to_string(),
                val: "2.3".to_string(),
            },
        ]];
        let mut s2r: Relation2StreamOperator<Vec<Binding>> =
            Relation2StreamOperator::new(ISTREAM, 0);
        s2r.eval(old_result, 1);

        assert_eq!(expected_deletion, s2r.eval(new_result, 2));
    }
