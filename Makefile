DOCKER_IMAGE=optimization_server
DOCKER_CONTAINER_NAME=optimization_server_test
DOCKER_RUN_CMD=docker run -d --name $(DOCKER_CONTAINER_NAME) --cpus="2" --memory="2g" -p 8080:8080 $(DOCKER_IMAGE)

init: ## Initialize Project
	@python3.12 -m venv venv
	@./venv/bin/python3 -m pip install pip --upgrade
	@./venv/bin/python3 -m pip install -U pip setuptools wheel pip-tools
	@./venv/bin/python3 -m pip install -r requirements/requirements.txt
	@./venv/bin/python3 -m pip install -r requirements/requirements-dev.txt
	@./venv/bin/python3 -m pip install -e . --no-deps
	@./venv/bin/python3 -m pre_commit install --install-hooks --overwrite

clean:  ## remove build artifacts
	rm -rf build dist venv pip-wheel-metadata htmlcov
	find . -name .tox | xargs rm -rf
	find . -name __pycache__ | xargs rm -rf
	find . -name .pytest_cache | xargs rm -rf
	find . -name *.egg-info | xargs rm -rf
	find . -name setup-py-dev-links | xargs rm -rf
	find docs -name generated | xargs rm -rf

update: clean init

lint: ## Run linters
	@./venv/bin/black --config=./pyproject.yoml .
	@./venv/bin/flake8 --config=./.flake8
	@./venv/bin/isort .

test: lint ## Run tests
	@./venv/bin/pytest -vv --durations=10 --cov-fail-under=90 --cov=portfolio-generator --cov-report html tests/

update-requirements: # Update requirements files from setup.py and requirements/requirements-dev.in
	./venv/bin/pip-compile setup.py --extra all requirements/constraints.in --strip-extras \
	--output-file=./requirements/requirements.txt --resolver=backtracking --verbose
	./venv/bin/pip-compile ./requirements/requirements-dev.in \
	--output-file=./requirements/requirements-dev.txt --resolver=backtracking --verbose

upgrade-requirements: # Upgrade requirements files from setup.py and requirements/requirements-dev.in
	./venv/bin/pip-compile setup.py --extra all requirements/constraints.in --upgrade --strip-extras \
	--output-file=./requirements/requirements.txt --upgrade --resolver=backtracking --verbose
	./venv/bin/pip-compile --upgrade ./requirements/requirements-dev.in \
	--output-file=./requirementsr/requirements-dev.txt --resolver=backtracking --verbose

reset-venv: # Makes installed packages in venv consistent with requirements
	./venv/bin/pip-sync ./requirements/requirements.txt ./requirements/requirements-dev.txt
	./venv/bin/pip install -e . --no-deps

sync-venv: update-requirements reset-venv ## Sync python environment deletes doc deps
	./venv/bin/pip-sync ./requirements/requirements.txt ./requirements/requirements-dev.txt
	./venv/bin/pip install -e . --no-deps

serve-docs: docs ## Serve docs in web-browser
	firefox docs/_build/html/index.html

load-test: ## Run load test
	# Build the docker image
	docker build -t $(DOCKER_IMAGE) -f docker/optimization_server/Dockerfile .
	# Bring up the container with resource constraints
	$(DOCKER_RUN_CMD)
	# Wait for the container to start
	sleep 5
	# Run the load test
	wrk -t12 -c400 -d30s --script tests/integration/load_testing.lua http://localhost:8080/optimize
	# Bring down the container
	docker stop $(DOCKER_CONTAINER_NAME)
	docker rm $(DOCKER_CONTAINER_NAME)