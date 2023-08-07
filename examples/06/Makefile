objs=$(shell find . -name '*.cc' | sed 's/\.cc/\.o/g')
CXXFLAGS=-Wall -Wextra -std=c++2b

test: $(objs)
	$(CXX) $(CXXFLAGS) $(objs) -o test

%.o: %.cc
	$(CXX) $(CXXFLAGS) -c $< -o $@

.PHONY: clean
clean:
	rm -f *.o test
