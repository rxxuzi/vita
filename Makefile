# Makefile
CC = gcc
CFLAGS = -Wall -Wextra -O2 -I$(SRCDIR)
LDFLAGS =

SRCDIR = src
BUILDDIR = build
BINDIR = .

C_SOURCES = $(wildcard $(SRCDIR)/*.c)
C_OBJECTS = $(patsubst $(SRCDIR)/%.c, $(BUILDDIR)/%.o, $(C_SOURCES))

# For Windows, adjust the TARGET extension if needed
ifeq ($(OS),Windows_NT)
    TARGET := vita.exe
else
    TARGET := vita
endif

all: $(BUILDDIR) $(TARGET)

$(BUILDDIR):
	mkdir -p $(BUILDDIR)

$(TARGET): $(C_OBJECTS)
	$(CC) $(CFLAGS) $(LDFLAGS) -o $(BINDIR)/$@ $^

$(BUILDDIR)/%.o: $(SRCDIR)/%.c
	$(CC) $(CFLAGS) -c $< -o $@

clean:
	rm -rf $(BUILDDIR) $(BINDIR)/$(TARGET)
