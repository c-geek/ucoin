use kv_typed::backend::memory::Mem;
use kv_typed::db_schema;
use kv_typed::prelude::*;
use std::collections::BTreeSet;

db_schema!(
    TestV1,
    [
        ["c1", Col1, i32, String],
        ["c2", Col2, usize, ()],
        ["c3", Col3, u64, Vec<u128>],
        ["c4", Col4, u64, BTreeSet<u128>],
    ]
);

#[test]
fn test_macro_db() {
    assert_eq!(Col1Event::RemoveAll, Col1Event::RemoveAll);

    #[cfg(feature = "explorer")]
    {
        use kv_typed::explorer::DbExplorable as _;
        assert_eq!(
            TestV1Db::<Mem>::list_collections(),
            vec![
                ("col1", "i32", "String"),
                ("col2", "usize", "()"),
                ("col3", "u64", "Vec<u128>"),
                ("col4", "u64", "BTreeSet<u128>")
            ]
        );
    }
}

#[test]
fn test_db_mem() -> KvResult<()> {
    let db = TestV1Db::<kv_typed::backend::memory::Mem>::open(
        kv_typed::backend::memory::MemConf::default(),
    )?;

    test_db(&db)
}

//#[cfg(feature = "sled_backend")]
#[test]
fn test_db_sled() -> KvResult<()> {
    let db = TestV1Db::<Sled>::open(SledConf::default().temporary(true))?;

    test_db(&db)
}

fn test_db<B: Backend>(db: &TestV1Db<B>) -> KvResult<()> {
    let (sender, recv) = kv_typed::channel::unbounded();
    db.col1().subscribe(sender)?;

    let db2 = db.clone();

    let handler = std::thread::spawn(move || db2.col1_write().upsert(3, "toto".to_owned()));
    handler.join().expect("thread panic")?;

    let expected_events: Events<Col1Event> = smallvec::smallvec![Col1Event::Upsert {
        key: 3,
        value: "toto".to_owned(),
    }];
    if let Ok(msg) = recv.recv() {
        assert_eq!(msg.as_ref(), &expected_events,)
    } else {
        panic!("must be receive event")
    }

    assert_eq!(db.col1().get(&3)?, Some("toto".to_owned()),);
    let d = db.col1().get_ref_slice(&3, |bytes| {
        let str_ = unsafe { core::str::from_utf8_unchecked(bytes) };
        assert_eq!("toto", str_);
        assert_eq!(db.col2().get(&3)?, None,);
        Ok(str_.to_owned())
    })?;
    assert_eq!(d, Some("toto".to_owned()));

    assert_eq!(db.col2().get(&3)?, None,);
    db.col2_write().upsert(3, ())?;
    assert_eq!(db.col2().get(&3)?, Some(()),);

    db.col1_write().upsert(5, "tutu".to_owned())?;

    db.col1().iter(.., |mut iter| {
        assert_eq!(iter.next_res()?, Some((3, "toto".to_owned())));
        assert_eq!(iter.next_res()?, Some((5, "tutu".to_owned())));
        assert_eq!(iter.next_res()?, None);
        Ok::<(), KvError>(())
    })?;

    db.col1().iter(.., |it| {
        let mut iter = it.values().reverse();

        assert_eq!(iter.next_res()?, Some("tutu".to_owned()));
        assert_eq!(iter.next_res()?, Some("toto".to_owned()));
        assert_eq!(iter.next_res()?, None);
        Ok::<(), KvError>(())
    })?;

    db.col1_write().upsert(7, "titi".to_owned())?;

    db.col1().iter(.., |it| {
        let mut iter = it.values().reverse().step_by(2);

        assert_eq!(iter.next_res()?, Some("titi".to_owned()));
        assert_eq!(iter.next_res()?, Some("toto".to_owned()));
        assert_eq!(iter.next_res()?, None);

        Ok::<(), KvError>(())
    })?;

    db.col3_write().upsert(4, vec![1, 2, 3])?;
    db.col3().get_ref_slice(&4, |numbers| {
        assert_eq!(numbers, &[1, 2, 3]);
        Ok(())
    })?;

    // Test get_ref_slice
    use std::iter::FromIterator as _;
    db.col4_write()
        .upsert(4, BTreeSet::from_iter((&[3, 2, 4, 1]).iter().copied()))?;
    db.col4().get_ref_slice(&4, |numbers| {
        assert_eq!(numbers, &[1, 2, 3, 4]);
        Ok(())
    })?;

    // Test transactional
    // A read tx should be opened when write tx not commited
    let (s1, r1) = flume::bounded::<()>(0);
    let (s2, r2) = flume::bounded::<()>(0);
    let db_ro = db.get_ro_handler();
    let read_task = std::thread::spawn(move || {
        r1.recv().expect("disconnected");
        (db_ro.col3(), db_ro.col4(), db_ro.col2()).read(|(c3, c4, _c2)| {
            c3.get_ref_slice(&4, |numbers| {
                assert_eq!(numbers, &[1, 2, 3]);
                Ok(())
            })?;
            c3.iter(.., |it| {
                let iter = it.keys();
                s2.send(()).expect("disconnected");
                assert_eq!(iter.collect::<KvResult<Vec<_>>>()?, vec![4]);
                Ok::<(), KvError>(())
            })?;
            c4.get_ref_slice(&4, |numbers| {
                assert_eq!(numbers, &[1, 2, 3, 4]);
                Ok(())
            })?;
            Ok(())
        })
    });

    let tres: KvResult<()> =
        (db.col3_write(), db.col4_write(), db.col2_write()).write(|(mut c3, mut c4, _c2)| {
            s1.send(()).expect("disconnected");
            assert_eq!(
                c3.iter(.., |it| it.keys().collect::<KvResult<Vec<_>>>())?,
                vec![4]
            );
            assert_eq!(
                c3.iter(.., |it| it.values().collect::<KvResult<Vec<_>>>())?,
                vec![vec![1, 2, 3]]
            );
            c3.upsert(42, vec![5, 4, 6]);
            assert_eq!(
                c3.iter(.., |it| it.keys().collect::<KvResult<Vec<_>>>())?,
                vec![4, 42]
            );
            assert_eq!(
                c3.iter(.., |it| it.reverse().keys().collect::<KvResult<Vec<_>>>())?,
                vec![42, 4]
            );
            c3.upsert(8, vec![11, 12, 13]);
            c3.remove(4);
            assert_eq!(
                c3.iter(.., |it| it.keys().collect::<KvResult<Vec<_>>>())?,
                vec![8, 42]
            );
            c3.iter(.., |it| {
                let iter = it.reverse().keys();
                r2.recv().expect("disconnected");
                assert_eq!(iter.collect::<KvResult<Vec<_>>>()?, vec![42, 8]);

                Ok::<(), KvError>(())
            })?;
            c4.upsert(4, BTreeSet::from_iter((&[7, 8, 6, 5]).iter().copied()));
            Ok(())
        });
    tres?;
    read_task.join().expect("read task panic")?;

    // Test clear()
    db.col4_write().clear()?;
    assert_eq!(db.col4().count()?, 0);

    Ok(())
}
