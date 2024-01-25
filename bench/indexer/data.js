window.BENCHMARK_DATA = {
  "lastUpdate": 1706209252610,
  "repoUrl": "https://github.com/MystenLabs/sui",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "email": "51927076+phoenix-o@users.noreply.github.com",
            "name": "phoenix",
            "username": "phoenix-o"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "704d061c82a9fe4a464612f98a58cee0260c27e4",
          "message": "[pruner] keep state of last manually processed SST files (#15921)\n\nPR introduces state tracking for the last manually processed SST files\r\nin memory. This prevents the recurrence of situations where the same\r\nfile is repeatedly selected for manual compaction because its initial\r\ncompaction run was a noop",
          "timestamp": "2024-01-25T13:53:14-05:00",
          "tree_id": "99675cd300aa6842ffb8d9b7bf1119628d422dc9",
          "url": "https://github.com/MystenLabs/sui/commit/704d061c82a9fe4a464612f98a58cee0260c27e4"
        },
        "date": 1706209248092,
        "tool": "cargo",
        "benches": [
          {
            "name": "get_checkpoint",
            "value": 325977,
            "range": "± 26985",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}