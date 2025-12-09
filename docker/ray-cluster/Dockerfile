FROM rayproject/ray:2.7.0-py310

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        python3-dev \
        unixodbc-dev \
        freetds-dev \
    && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir \
    connectorx>=0.3.2 \
    pymssql>=2.2.10 \
    sqlalchemy>=2.0

RUN python - <<'PY'
import connectorx as cx
import sqlite3
import tempfile
import os
import pymssql

print(f"connectorx version: {cx.__version__}")

tmp = tempfile.NamedTemporaryFile(suffix=".db", delete=False)
path = tmp.name
tmp.close()

conn = sqlite3.connect(path)
conn.execute("create table t(x int)")
conn.execute("insert into t values (1)")
conn.commit()
conn.close()

frame = cx.read_sql(f"sqlite:///{path}", "select x from t")
assert frame.iloc[0, 0] == 1
print("connectorx read_sql against sqlite succeeded", frame.to_dict())

os.remove(path)
print("pymssql import succeeded", pymssql.__version__)
PY
