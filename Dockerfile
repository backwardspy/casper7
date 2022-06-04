FROM python:3.10 AS build

RUN pip install poetry==1.2.0b1
RUN poetry config virtualenvs.create false

ADD . .
RUN poetry build

FROM python:3.10

COPY --from=build dist/casper7-0.0.0-py3-none-any.whl .
RUN pip install casper7-0.0.0-py3-none-any.whl && rm casper7-0.0.0-py3-none-any.whl
RUN pip install casper7-plugin-meatball-day

CMD casper7
