use lazy_static::lazy_static;
use mongodb::bson::doc;

lazy_static! {
    pub static ref POPULATE_BLOCKS_AGGREGATION: [mongodb::bson::Document; 8] = [
        doc! {
            "$sort": doc! {
                "startTime": 1
            }
        },
        doc! {
            "$lookup": doc! {
                "from": "users",
                "localField": "blue1",
                "foreignField": "_id",
                "as": "blue1"
            }
        },
        doc! {
            "$lookup": doc! {
                "from": "users",
                "localField": "blue2",
                "foreignField": "_id",
                "as": "blue2"
            }
        },
        doc! {
            "$lookup": doc! {
                "from": "users",
                "localField": "blue3",
                "foreignField": "_id",
                "as": "blue3"
            }
        },
        doc! {
            "$lookup": doc! {
                "from": "users",
                "localField": "red1",
                "foreignField": "_id",
                "as": "red1"
            }
        },
        doc! {
            "$lookup": doc! {
                "from": "users",
                "localField": "red2",
                "foreignField": "_id",
                "as": "red2"
            }
        },
        doc! {
            "$lookup": doc! {
                "from": "users",
                "localField": "red3",
                "foreignField": "_id",
                "as": "red3"
            }
        },
        doc! {
            "$project": doc! {
                "startMatch": true,
                "lastMatch": true,
                "threeAway": true,
                "oneAway": true,
                "createdAt": true,
                "updatedAt": true,
                "blue1": doc! {
                    "$first": "$blue1"
                },
                "blue2": doc! {
                    "$first": "$blue2"
                },
                "blue3": doc! {
                    "$first": "$blue3"
                },
                "red1": doc! {
                    "$first": "$red1"
                },
                "red2": doc! {
                    "$first": "$red2"
                },
                "red3": doc! {
                    "$first": "$red3"
                }
            }
        }
    ];
}
