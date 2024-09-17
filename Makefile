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

rebuild-notebooks: ## Re-run notebooks for latest outputs
	./venv/bin/python3 src/docs/run-notebooks.py; \
	username=`whoami`; \
	date; \
	for f in $(shell find notebooks/ -type f -name "*.ipynb" | sort); do \
		echo "Scrub username from '$${f}'"; \
		sed -i"" "s/$${username}/jdoe/g" $${f}; \
		date; \
	done

docs-build: ## Build docs
	# run sphinx to build docs
	./venv/bin/sphinx-build -c docs/ -w docs.log docs/ docs/_build/html/
	mkdir -p docs/_build/html/_static/notebooks
	cp notebooks/*.ipynb docs/_build/html/_static/notebooks

docs: rebuild-notebooks docs-build ## Build documentation and API docs

serve-docs: docs ## Serve docs in web-browser
	firefox docs/_build/html/index.html

serve-django: ## Load Django App
	@./venv/bin/python3 app/manage.py runserver

build-docker-aws: ## Build docker image
	docker build --progress=plain \
		--secret id=ssh-private-key,src=${HOME}/.ssh/id_ed25519 \
		-t 516823451519.dkr.ecr.us-east-1.amazonaws.com/medication-extractor-repo:latest .
	docker push 516823451519.dkr.ecr.us-east-1.amazonaws.com/medication-extractor-repo:latest

build-docker-local: ## Build docker image
	docker build --progress=plain \
		--secret id=ssh-private-key,src=${HOME}/.ssh/id_ed25519 \
		-t medication-extractor:local .

run-docker: ## Run docker image
	docker run -it \
		-p 8000:8000 \
		-v ~/.aws/credentials:/root/.aws/credentials:ro \
		-v ~/.aws/config:/root/.aws/config:ro \
		medication-extractor:local